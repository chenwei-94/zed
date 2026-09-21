use anyhow::{Context as _, Result, bail};
use async_trait::async_trait;
use collections::HashMap;
use dap::{StartDebuggingRequestArguments, adapters::DebugTaskDefinition};
use gpui::AsyncApp;
use std::ffi::OsStr;
use task::{DebugScenario, ZedDebugConfig};

use crate::*;

#[derive(Default)]
pub(crate) struct GdbDebugAdapter;

impl GdbDebugAdapter {
    const ADAPTER_NAME: &'static str = "GDB";
}

/// Ensures that "-i=dap" is present in the GDB argument list.
fn ensure_dap_interface(mut gdb_args: Vec<String>) -> Vec<String> {
    if !gdb_args.iter().any(|arg| arg.trim() == "-i=dap") {
        gdb_args.insert(0, "-i=dap".to_string());
    }
    gdb_args
}

#[async_trait(?Send)]
impl DebugAdapter for GdbDebugAdapter {
    fn name(&self) -> DebugAdapterName {
        DebugAdapterName(Self::ADAPTER_NAME.into())
    }

    async fn config_from_zed_format(&self, zed_scenario: ZedDebugConfig) -> Result<DebugScenario> {
        let mut obj = serde_json::Map::default();

        match &zed_scenario.request {
            dap::DebugRequest::Attach(attach) => {
                obj.insert("request".into(), "attach".into());
                obj.insert("pid".into(), attach.process_id.into());
            }

            dap::DebugRequest::Launch(launch) => {
                obj.insert("request".into(), "launch".into());
                obj.insert("program".into(), launch.program.clone().into());

                if !launch.args.is_empty() {
                    obj.insert("args".into(), launch.args.clone().into());
                }

                if !launch.env.is_empty() {
                    obj.insert("env".into(), launch.env_json());
                }

                if let Some(stop_on_entry) = zed_scenario.stop_on_entry {
                    obj.insert(
                        "stopAtBeginningOfMainSubprogram".into(),
                        stop_on_entry.into(),
                    );
                }
                if let Some(cwd) = launch.cwd.as_ref() {
                    obj.insert("cwd".into(), cwd.to_string_lossy().into_owned().into());
                }
            }
        }

        Ok(DebugScenario {
            adapter: zed_scenario.adapter,
            label: zed_scenario.label,
            build: None,
            config: serde_json::Value::Object(obj),
            tcp_connection: None,
        })
    }

    fn dap_schema(&self) -> serde_json::Value {
        json!({
            "oneOf": [
                {
                    "allOf": [
                        {
                            "type": "object",
                            "required": ["request"],
                            "properties": {
                                "request": {
                                    "type": "string",
                                    "enum": ["launch"],
                                    "description": "请求启动新进程"
                                }
                            }
                        },
                        {
                            "type": "object",
                            "properties": {
                                "program": {
                                    "type": "string",
                                    "description": "要调试的程序。对应 GDB 的 'file' 命令。"
                                },
                                "args": {
                                    "type": "array",
                                    "items": {
                                        "type": "string"
                                    },
                                    "description": "传递给程序的命令行参数。这些字符串会作为命令行参数提供给被调试进程。",
                                    "default": []
                                },
                                "cwd": {
                                    "type": "string",
                                    "description": "被调试程序的工作目录。GDB 会将其工作目录切换到该目录。"
                                },
                                "gdb_path": {
                                    "type": "string",
                                    "description": "GDB 可执行文件的替代路径，若标准路径中的那个不理想"
                                },
                                "gdb_args": {
                                    "type": "array",
                                    "items": {
                                        "type":"string"
                                    },
                                    "description": "启动时传给 GDB 的额外参数，而非被调试程序",
                                    "default": []
                                },
                                "env": {
                                    "type": "object",
                                    "description": "被调试程序的环境变量。每个键是环境变量的名称；每个值是那个变量的值。"
                                },
                                "stopAtBeginningOfMainSubprogram": {
                                    "type": "boolean",
                                    "description": "当为 true 时，GDB 会像 'start' 命令那样在程序主过程处设置临时断点。",
                                    "default": false
                                },
                                "stopOnEntry": {
                                    "type": "boolean",
                                    "description": "当为 true 时，GDB 会像 'starti' 命令那样在程序第一条指令处设置临时断点。",
                                    "default": false
                                }
                            },
                            "required": ["program"]
                        }
                    ]
                },
                {
                    "allOf": [
                        {
                            "type": "object",
                            "required": ["request"],
                            "properties": {
                                "request": {
                                    "type": "string",
                                    "enum": ["attach"],
                                    "description": "请求附加到现有进程"
                                }
                            }
                        },
                        {
                            "type": "object",
                            "properties": {
                                "pid": {
                                    "type": "number",
                                    "description": "GDB 应附加到的进程 ID。"
                                },
                                "program": {
                                    "type": "string",
                                    "description": "要调试的程序（可选）。对应 GDB 的 'file' 命令。多数情况下 GDB 能自动判断正在运行的程序。"
                                },
                                "target": {
                                    "type": "string",
                                    "description": "GDB 应连接的目标。此参数会传给 'target remote' 命令。"
                                }
                            },
                            "required": ["pid"]
                        }
                    ]
                }
            ]
        })
    }

    async fn get_binary(
        &self,
        delegate: &Arc<dyn DapDelegate>,
        config: &DebugTaskDefinition,
        user_installed_path: Option<std::path::PathBuf>,
        user_args: Option<Vec<String>>,
        user_env: Option<HashMap<String, String>>,
        _: &mut AsyncApp,
    ) -> Result<DebugAdapterBinary> {
        // Try to get gdb_path from config
        let gdb_path_from_config = config
            .config
            .get("gdb_path")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let gdb_path = if let Some(path) = gdb_path_from_config {
            path
        } else {
            // Original logic: use user_installed_path or search in system path
            let user_setting_path = user_installed_path
                .filter(|p| p.exists())
                .and_then(|p| p.to_str().map(|s| s.to_string()));

            let gdb_path_result = delegate
                .which(OsStr::new("gdb"))
                .await
                .and_then(|p| p.to_str().map(|s| s.to_string()))
                .context("Could not find gdb in path");

            if gdb_path_result.is_err() && user_setting_path.is_none() {
                bail!("Could not find gdb path or it's not installed");
            }

            user_setting_path.unwrap_or_else(|| gdb_path_result.unwrap())
        };

        // Arguments: use gdb_args from config if present, else user_args, else default
        let gdb_args = {
            let args = config
                .config
                .get("gdb_args")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                        .collect::<Vec<_>>()
                })
                .or(user_args.clone())
                .unwrap_or_else(|| vec!["-i=dap".into()]);
            ensure_dap_interface(args)
        };

        let mut configuration = config.config.clone();
        if let Some(configuration) = configuration.as_object_mut() {
            configuration
                .entry("cwd")
                .or_insert_with(|| delegate.worktree_root_path().to_string_lossy().into());
        }

        let mut base_env = delegate.shell_env().await;
        base_env.extend(user_env.unwrap_or_default());

        let config_env: HashMap<String, String> = config
            .config
            .get("env")
            .and_then(|v| v.as_object())
            .map(|obj| {
                obj.iter()
                    .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                    .collect::<HashMap<String, String>>()
            })
            .unwrap_or_else(HashMap::default);

        base_env.extend(config_env);

        Ok(DebugAdapterBinary {
            command: Some(gdb_path),
            arguments: gdb_args,
            envs: base_env,
            cwd: Some(delegate.worktree_root_path().to_path_buf()),
            connection: None,
            request_args: StartDebuggingRequestArguments {
                request: self.request_kind(&config.config).await?,
                configuration,
            },
        })
    }
}
