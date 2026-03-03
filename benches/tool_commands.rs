mod support;

use std::time::Duration;

use criterion::{BatchSize, Criterion, black_box, criterion_group, criterion_main};
use serde_json::json;

#[cfg(unix)]
use std::path::Path;
#[cfg(unix)]
use std::process::Command;

use crate::support::{
	PATCH_TEXT, build_services, dispatch_ok, runtime_log_relative_path, seed_memory_fact,
	seed_patch_target, start_bulk_background, write_log_fixture, write_plan_fixture,
};

#[cfg(unix)]
fn run_os_command(working_dir: &Path, program: &str, args: &[&str]) -> Vec<u8> {
	let output = Command::new(program)
		.args(args)
		.current_dir(working_dir)
		.output()
		.unwrap_or_else(|error| panic!("failed to execute {program}: {error}"));
	if !output.status.success() {
		panic!(
			"command {program} failed with status {:?}: stderr={} args={args:?}",
			output.status.code(),
			String::from_utf8_lossy(&output.stderr)
		);
	}
	output.stdout
}

fn bench_top_level_commands(c: &mut Criterion) {
	let (services, workspace) = build_services("bench-top-level", true);
	write_log_fixture(&services);
	write_plan_fixture(&services);
	seed_patch_target(&services);

	// Explicit one-time warmup for index-backed/stateful paths.
	let _ = dispatch_ok(
		"search",
		json!({"query": "Hello from", "mode": "literal", "limit": 10}),
		&services,
	);
	let _ = dispatch_ok("status", json!({"verbose": true}), &services);

	let mut group = c.benchmark_group("tool_commands/top_level");
	group.bench_function("workspace_dir", |b| {
		b.iter(|| {
			let data = dispatch_ok("workspace_dir", json!({}), &services);
			black_box(data);
		})
	});
	group.bench_function("capabilities", |b| {
		b.iter(|| {
			let data = dispatch_ok("capabilities", json!({}), &services);
			black_box(data);
		})
	});
	group.bench_function("search", |b| {
		b.iter(|| {
			let data = dispatch_ok(
				"search",
				json!({"query": "Hello from", "mode": "literal", "limit": 10}),
				&services,
			);
			black_box(data);
		})
	});
	group.bench_function("status", |b| {
		b.iter(|| {
			let data = dispatch_ok("status", json!({"verbose": true}), &services);
			black_box(data);
		})
	});
	group.finish();
	black_box(&workspace.path);
}

fn bench_fs_subcommands(c: &mut Criterion) {
	let (services, workspace) = build_services("bench-fs", false);
	let _ = dispatch_ok(
		"fs",
		json!({"operation": "search", "args": {"path": ".", "recursive": false}}),
		&services,
	);
	let _ = dispatch_ok(
		"fs",
		json!({"operation": "read_range", "args": {"path": "alpha.txt", "start_line": 1, "end_line": 1}}),
		&services,
	);

	let mut group = c.benchmark_group("tool_commands/fs");

	let fs_calls = vec![
		(
			"search",
			json!({"operation": "search", "args": {"path": ".", "recursive": false}}),
		),
		(
			"read_range",
			json!({"operation": "read_range", "args": {"path": "alpha.txt", "start_line": 1, "end_line": 1}}),
		),
		(
			"stat",
			json!({"operation": "stat", "args": {"path": "alpha.txt"}}),
		),
		(
			"diff",
			json!({
				"operation": "diff",
				"args": {
					"a": {"path": "alpha.txt", "from": 1, "to": 1},
					"b": {"path": "nested/beta.md", "from": 1, "to": 1}
				}
			}),
		),
		(
			"create_file",
			json!({"operation": "create_file", "args": {"path": "created.txt", "content": "hello"}}),
		),
		(
			"append_file",
			json!({"operation": "append_file", "args": {"path": "alpha.txt", "content": " world"}}),
		),
		(
			"delete_file",
			json!({"operation": "delete_file", "args": {"path": "alpha.txt"}}),
		),
		(
			"create_dir",
			json!({"operation": "create_dir", "args": {"path": "bench-dir", "parents": true}}),
		),
		(
			"delete_dir",
			json!({"operation": "delete_dir", "args": {"path": "nested"}}),
		),
		(
			"bulk.execute",
			json!({
				"operation": "bulk",
				"dry_run": true,
				"args": {
					"mode": "execute",
					"operations": [{"kind": "delete", "path": "nested/beta.md", "recursive": false}]
				}
			}),
		),
	];

	for (name, args) in fs_calls {
		let id = format!("fs.{name}");
		group.bench_function(id, |b| {
			b.iter(|| {
				let data = dispatch_ok("fs", args.clone(), &services);
				black_box(data);
			})
		});
	}

	group.bench_function("fs.bulk.status", |b| {
		b.iter_batched(
			|| start_bulk_background(&services),
			|operation_id| {
				let data = dispatch_ok(
					"fs",
					json!({"operation": "bulk", "args": {"mode": "status", "operation_id": operation_id}}),
					&services,
				);
				black_box(data);
			},
			BatchSize::SmallInput,
		)
	});

	group.bench_function("fs.bulk.cancel", |b| {
		b.iter_batched(
			|| start_bulk_background(&services),
			|operation_id| {
				let data = dispatch_ok(
					"fs",
					json!({"operation": "bulk", "args": {"mode": "cancel", "operation_id": operation_id}}),
					&services,
				);
				black_box(data);
			},
			BatchSize::SmallInput,
		)
	});

	group.finish();
	black_box(&workspace.path);
}

fn bench_logs_subcommands(c: &mut Criterion) {
	let (services, workspace) = build_services("bench-logs", false);
	write_log_fixture(&services);
	let _ = dispatch_ok(
		"logs",
		json!({"operation": "search", "args": {"query": "build", "limit": 10}}),
		&services,
	);
	let _ = services.logs.stop_follow();

	let mut group = c.benchmark_group("tool_commands/logs");
	group.bench_function("logs.search", |b| {
		b.iter(|| {
			let data = dispatch_ok(
				"logs",
				json!({"operation": "search", "args": {"query": "build", "limit": 10}}),
				&services,
			);
			black_box(data);
		})
	});
	group.bench_function("logs.tail", |b| {
		b.iter(|| {
			let data = dispatch_ok(
				"logs",
				json!({"operation": "tail", "args": {"limit": 10}}),
				&services,
			);
			black_box(data);
		})
	});
	group.bench_function("logs.follow.start", |b| {
		b.iter_batched(
			|| {
				let _ = services.logs.stop_follow();
			},
			|_| {
				let data = dispatch_ok(
					"logs",
					json!({"operation": "follow", "args": {"tail": 1}}),
					&services,
				);
				black_box(data);
			},
			BatchSize::PerIteration,
		)
	});
	group.bench_function("logs.follow.stop", |b| {
		b.iter_batched(
			|| {
				let _ = services.logs.stop_follow();
				dispatch_ok(
					"logs",
					json!({"operation": "follow", "args": {"tail": 1}}),
					&services,
				)
			},
			|_| {
				let data = dispatch_ok(
					"logs",
					json!({"operation": "follow", "args": {"stop": true}}),
					&services,
				);
				black_box(data);
			},
			BatchSize::PerIteration,
		)
	});
	group.finish();
	black_box(&workspace.path);
}

fn bench_patch_subcommands(c: &mut Criterion) {
	let (services, workspace) = build_services("bench-patch", false);
	seed_patch_target(&services);
	let _ = dispatch_ok(
		"patch",
		json!({"operation": "apply", "dry_run": true, "patches": [{"path": "hello.txt", "patch": PATCH_TEXT}]}),
		&services,
	);

	let mut group = c.benchmark_group("tool_commands/patch");
	group.bench_function("patch.apply", |b| {
		b.iter(|| {
			let data = dispatch_ok(
				"patch",
				json!({"operation": "apply", "dry_run": true, "patches": [{"path": "hello.txt", "patch": PATCH_TEXT}]}),
				&services,
			);
			black_box(data);
		})
	});
	group.bench_function("patch.revert", |b| {
		b.iter_batched(
			|| {
				seed_patch_target(&services);
				dispatch_ok(
					"patch",
					json!({"operation": "apply", "dry_run": false, "patches": [{"path": "hello.txt", "patch": PATCH_TEXT}]}),
					&services,
				)
			},
			|_| {
				let data = dispatch_ok("patch", json!({"operation": "revert", "dry_run": true}), &services);
				black_box(data);
			},
			BatchSize::SmallInput,
		)
	});
	group.finish();
	black_box(&workspace.path);
}

fn bench_plan_subcommands(c: &mut Criterion) {
	let (services, workspace) = build_services("bench-plan", false);
	write_plan_fixture(&services);
	let _ = dispatch_ok("plan", json!({"operation": "get", "args": {}}), &services);

	let mut group = c.benchmark_group("tool_commands/plan");
	group.bench_function("plan.get", |b| {
		b.iter(|| {
			let data = dispatch_ok("plan", json!({"operation": "get", "args": {}}), &services);
			black_box(data);
		})
	});
	group.bench_function("plan.add", |b| {
		b.iter_batched(
			|| write_plan_fixture(&services),
			|_| {
				let data = dispatch_ok(
					"plan",
					json!({
						"operation": "add",
						"args": {
							"title": "Bench task",
							"priority": 1,
							"cards": ["ART-BENCH"],
							"description": "Benchmark add",
							"deliverables": ["Benchmark output"],
							"status": "planned"
						}
					}),
					&services,
				);
				black_box(data);
			},
			BatchSize::PerIteration,
		)
	});
	group.bench_function("plan.update_status", |b| {
		b.iter_batched(
			|| write_plan_fixture(&services),
			|_| {
				let data = dispatch_ok(
					"plan",
					json!({"operation": "update_status", "args": {"id": 1, "status": "in-progress"}}),
					&services,
				);
				black_box(data);
			},
			BatchSize::PerIteration,
		)
	});
	group.bench_function("plan.delete", |b| {
		b.iter_batched(
			|| write_plan_fixture(&services),
			|_| {
				let data = dispatch_ok(
					"plan",
					json!({"operation": "delete", "args": {"id": 1}}),
					&services,
				);
				black_box(data);
			},
			BatchSize::PerIteration,
		)
	});
	group.finish();
	black_box(&workspace.path);
}

fn bench_memory_subcommands(c: &mut Criterion) {
	let (services, workspace) = build_services("bench-memory", false);
	let memory_id = seed_memory_fact(&services);
	let _ = dispatch_ok(
		"memory",
		json!({"operation": "search", "args": {"query": "benchmark", "limit": 10}}),
		&services,
	);

	let mut group = c.benchmark_group("tool_commands/memory");
	group.bench_function("memory.create", |b| {
		b.iter(|| {
			let data = dispatch_ok(
				"memory",
				json!({
					"operation": "create",
					"args": {
						"scope": "user",
						"subject": "Benchmark create",
						"category": "general",
						"fact": "create benchmark",
						"reasoning": "benchmark",
						"tags": ["bench", "create"]
					}
				}),
				&services,
			);
			black_box(data);
		})
	});
	group.bench_function("memory.retrieve", |b| {
		b.iter(|| {
			let data = dispatch_ok(
				"memory",
				json!({"operation": "retrieve", "args": {"id": memory_id}}),
				&services,
			);
			black_box(data);
		})
	});
	group.bench_function("memory.update", |b| {
		b.iter(|| {
			let data = dispatch_ok(
				"memory",
				json!({
					"operation": "update",
					"args": {
						"id": memory_id,
						"scope": "user",
						"subject": "Benchmark subject",
						"category": "general",
						"fact": "Benchmark fact updated",
						"reasoning": "benchmark update",
						"tags": ["bench", "seed"]
					}
				}),
				&services,
			);
			black_box(data);
		})
	});
	group.bench_function("memory.delete", |b| {
		b.iter(|| {
			let data = dispatch_ok(
				"memory",
				json!({"operation": "delete", "args": {"id": memory_id, "dry_run": true}}),
				&services,
			);
			black_box(data);
		})
	});
	group.bench_function("memory.search", |b| {
		b.iter(|| {
			let data = dispatch_ok(
				"memory",
				json!({"operation": "search", "args": {"query": "benchmark", "limit": 10}}),
				&services,
			);
			black_box(data);
		})
	});
	group.finish();
	black_box(&workspace.path);
}

#[cfg(unix)]
fn bench_os_baselines(c: &mut Criterion) {
	let (services, workspace) = build_services("bench-os-baseline", true);
	write_log_fixture(&services);

	let workspace_root = workspace.path.as_path();
	let log_path = runtime_log_relative_path(&services);

	let mut group = c.benchmark_group("tool_commands/os_baseline");
	group.bench_function("search.grep_recursive", |b| {
		b.iter(|| {
			let output = run_os_command(workspace_root, "grep", &["-R", "-n", "Hello from", "."]);
			black_box(output);
		})
	});
	group.bench_function("fs.search.find", |b| {
		b.iter(|| {
			let output = run_os_command(workspace_root, "find", &[".", "-maxdepth", "2"]);
			black_box(output);
		})
	});
	group.bench_function("fs.read_range.head", |b| {
		b.iter(|| {
			let output = run_os_command(workspace_root, "head", &["-n", "1", "alpha.txt"]);
			black_box(output);
		})
	});
	group.bench_function("logs.search.grep", |b| {
		b.iter(|| {
			let output =
				run_os_command(workspace_root, "grep", &["-n", "build", log_path.as_str()]);
			black_box(output);
		})
	});
	group.bench_function("logs.tail.tail", |b| {
		b.iter(|| {
			let output = run_os_command(workspace_root, "tail", &["-n", "10", log_path.as_str()]);
			black_box(output);
		})
	});
	group.finish();
}

criterion_group! {
	name = command_benches;
	config = Criterion::default()
		.warm_up_time(Duration::from_secs(1))
		.measurement_time(Duration::from_secs(2))
		.sample_size(20);
	targets =
		bench_top_level_commands,
		bench_fs_subcommands,
		bench_logs_subcommands,
		bench_patch_subcommands,
		bench_plan_subcommands,
		bench_memory_subcommands,
		bench_os_baselines
}
criterion_main!(command_benches);
