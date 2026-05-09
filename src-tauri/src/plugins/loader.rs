//! 插件加载器模块
//!
//! 从 plugins/ 目录加载 JSON 格式的插件配置，
//! 动态注册工具和扩展功能。

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// 插件元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMeta {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
}

/// 插件定义（JSON 文件格式）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginDefinition {
    pub meta: PluginMeta,
    /// 插件提供的工具列表
    #[serde(default)]
    pub tools: Vec<PluginTool>,
    /// 插件提供的命令列表
    #[serde(default)]
    pub commands: Vec<PluginCommand>,
    /// 插件是否启用
    #[serde(default = "default_true")]
    pub enabled: bool,
}

fn default_true() -> bool {
    true
}

/// 插件工具定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginTool {
    pub name: String,
    pub description: String,
    /// JSON Schema 格式的参数定义
    #[serde(default)]
    pub parameters: serde_json::Value,
    /// 要执行的命令（shell command）
    pub command: String,
}

/// 插件命令定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginCommand {
    pub name: String,
    pub description: String,
    pub command: String,
}

/// 插件加载器
pub struct PluginLoader {
    plugins_dir: PathBuf,
    plugins: Vec<PluginDefinition>,
}

impl PluginLoader {
    /// 创建新的插件加载器
    pub fn new(plugins_dir: PathBuf) -> Self {
        Self {
            plugins_dir,
            plugins: Vec::new(),
        }
    }

    /// 从插件目录加载所有插件
    pub fn load_all(&mut self) -> Result<usize, String> {
        self.plugins.clear();

        if !self.plugins_dir.exists() {
            std::fs::create_dir_all(&self.plugins_dir)
                .map_err(|e| format!("Failed to create plugins dir: {}", e))?;
            return Ok(0);
        }

        let entries = std::fs::read_dir(&self.plugins_dir)
            .map_err(|e| format!("Failed to read plugins dir: {}", e))?;

        let mut count = 0;
        for entry in entries {
            let entry = entry.map_err(|e| format!("Dir entry error: {}", e))?;
            let path = entry.path();

            if path.extension().and_then(|e| e.to_str()) == Some("json") {
                match self.load_plugin(&path) {
                    Ok(plugin) => {
                        if plugin.enabled {
                            eprintln!(
                                "[plugin] Loaded: {} v{}",
                                plugin.meta.name,
                                plugin.meta.version
                            );
                            self.plugins.push(plugin);
                            count += 1;
                        }
                    }
                    Err(e) => {
                        eprintln!(
                            "[plugin] Failed to load {:?}: {}",
                            path.file_name(),
                            e
                        );
                    }
                }
            }
        }

        Ok(count)
    }

    /// 加载单个插件文件
    fn load_plugin(&self, path: &std::path::Path) -> Result<PluginDefinition, String> {
        let content =
            std::fs::read_to_string(path).map_err(|e| format!("Read error: {}", e))?;
        let plugin: PluginDefinition =
            serde_json::from_str(&content).map_err(|e| format!("Parse error: {}", e))?;
        Ok(plugin)
    }

    /// 获取所有插件的工具列表
    pub fn get_tools(&self) -> Vec<&PluginTool> {
        self.plugins.iter().flat_map(|p| p.tools.iter()).collect()
    }

    /// 获取所有插件
    pub fn get_all(&self) -> &[PluginDefinition] {
        &self.plugins
    }

    /// 按名称查找插件
    pub fn find(&self, name: &str) -> Option<&PluginDefinition> {
        self.plugins.iter().find(|p| p.meta.name == name)
    }

    /// 获取插件数量
    pub fn count(&self) -> usize {
        self.plugins.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_serialization() {
        let plugin = PluginDefinition {
            meta: PluginMeta {
                name: "test".into(),
                version: "0.1.0".into(),
                description: "test plugin".into(),
                author: "test".into(),
            },
            tools: vec![],
            commands: vec![],
            enabled: true,
        };
        let json = serde_json::to_string_pretty(&plugin).unwrap();
        let parsed: PluginDefinition = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.meta.name, "test");
    }

    #[test]
    fn test_plugin_loader_empty_dir() {
        let dir = std::env::temp_dir().join("fairyfield_test_plugins_empty");
        let _ = std::fs::remove_dir_all(&dir);
        let mut loader = PluginLoader::new(dir.clone());
        let count = loader.load_all().unwrap();
        assert_eq!(count, 0);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_plugin_loader_loads_plugin() {
        let dir = std::env::temp_dir().join("fairyfield_test_plugins_load");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        let plugin_json = r#"{
            "meta": {
                "name": "hello",
                "version": "1.0.0",
                "description": "A test plugin",
                "author": "test"
            },
            "tools": [],
            "commands": [],
            "enabled": true
        }"#;

        std::fs::write(dir.join("hello.json"), plugin_json).unwrap();

        let mut loader = PluginLoader::new(dir.clone());
        let count = loader.load_all().unwrap();
        assert_eq!(count, 1);
        assert_eq!(loader.count(), 1);

        let plugin = loader.find("hello").unwrap();
        assert_eq!(plugin.meta.version, "1.0.0");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn test_plugin_loader_skips_disabled() {
        let dir = std::env::temp_dir().join("fairyfield_test_plugins_disabled");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        let plugin_json = r#"{
            "meta": {
                "name": "disabled_plugin",
                "version": "0.1.0",
                "description": "Should be skipped",
                "author": "test"
            },
            "tools": [],
            "commands": [],
            "enabled": false
        }"#;

        std::fs::write(dir.join("disabled.json"), plugin_json).unwrap();

        let mut loader = PluginLoader::new(dir.clone());
        let count = loader.load_all().unwrap();
        assert_eq!(count, 0);

        let _ = std::fs::remove_dir_all(&dir);
    }
}
