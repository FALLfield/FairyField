# Live2D 模型目录 / Live2D Model Directory

## 概述 / Overview

此目录用于存放 FairyField 应用的默认 Live2D 模型文件。
This directory is for storing the default Live2D model files for the FairyField application.

## 所需文件 / Required Files

一个完整的 Live2D Cubism 3.x/4.x 模型需要以下文件：
A complete Live2D Cubism 3.x/4.x model requires the following files:

```
public/models/default/
├── model3.json          # 模型配置文件 / Model configuration file
├── *.moc3               # 模型数据文件 / Model data file
├── textures/            # 纹理目录 / Textures directory
│   ├── texture_00.png
│   └── ...
├── motions/             # 动作目录 / Motions directory (可选 / optional)
│   ├── idle_01.motion3.json
│   └── ...
├── expressions/         # 表情目录 / Expressions directory (可选 / optional)
│   ├── happy.exp3.json
│   └── ...
├── physics3.json        # 物理配置 / Physics configuration (可选 / optional)
└── pose3.json           # 姿势配置 / Pose configuration (可选 / optional)
```

## 如何获取 Live2D 模型 / How to Obtain Live2D Models

### 选项 1: 官方示例模型 / Option 1: Official Sample Models

从 Live2D 官方 Cubism SDK 下载示例模型：
Download sample models from the official Live2D Cubism SDK:

1. 访问 / Visit: https://www.live2d.com/en/download/cubism-sdk/
2. 下载 "Cubism SDK for Web" / Download "Cubism SDK for Web"
3. 解压后，在 `Samples/Resources/` 目录中找到示例模型（如 Haru, Hiyori, Mark 等）
   After extraction, find sample models in `Samples/Resources/` directory (e.g., Haru, Hiyori, Mark)
4. 将整个模型文件夹的内容复制到此目录
   Copy the entire model folder contents to this directory

**注意 / Note**: 官方示例模型受 Live2D 的免费素材许可证约束，请查看许可证条款。
Official sample models are subject to Live2D's Free Material License. Please review the license terms.

### 选项 2: 免费社区模型 / Option 2: Free Community Models

以下是一些提供免费 Live2D 模型的资源：
Here are some resources that provide free Live2D models:

1. **itch.io**
   - Free Slime Model: https://rhapsperhaps.itch.io/free-slime-model
   - Little Cat Model: https://ezrii.itch.io/live2d-little-cat-model

2. **BOOTH (日本平台 / Japanese platform)**
   - 搜索 "Live2D 無料" / Search for "Live2D 無料"
   - https://booth.pm/

3. **GitHub 社区项目 / GitHub Community Projects**
   - 搜索 "live2d model" 标签 / Search for "live2d model" topic
   - https://github.com/topics/live2d

**重要 / Important**: 使用任何模型前，请务必检查其许可证和使用条款。
Always check the license and terms of use before using any model.

### 选项 3: 创建自己的模型 / Option 3: Create Your Own Model

使用 Live2D Cubism Editor 创建自定义模型：
Create a custom model using Live2D Cubism Editor:

1. 下载 Live2D Cubism Editor / Download Live2D Cubism Editor
   - https://www.live2d.com/en/download/cubism/
2. 学习教程 / Learn tutorials
   - 官方教程 / Official tutorials: https://docs.live2d.com/
3. 导出为 .model3.json 格式 / Export as .model3.json format

## 模型配置示例 / Model Configuration Example

一个典型的 `model3.json` 文件结构：
A typical `model3.json` file structure:

```json
{
  "Version": 3,
  "FileReferences": {
    "Moc": "model.moc3",
    "Textures": [
      "textures/texture_00.png"
    ],
    "Physics": "physics3.json",
    "Pose": "pose3.json",
    "Expressions": [
      {
        "Name": "happy",
        "File": "expressions/happy.exp3.json"
      }
    ],
    "Motions": {
      "Idle": [
        {
          "File": "motions/idle_01.motion3.json"
        }
      ]
    }
  },
  "Groups": [],
  "HitAreas": []
}
```

## 验证模型 / Verify Model

安装模型后，可以通过以下方式验证：
After installing a model, you can verify it by:

1. 运行验证脚本 / Run verification script:
   ```bash
   node verify-model.cjs
   ```
2. 确保 `model3.json` 文件存在 / Ensure `model3.json` file exists
3. 检查所有引用的文件路径是否正确 / Check all referenced file paths are correct
4. 运行 FairyField 应用并查看控制台是否有错误
   Run the FairyField application and check the console for errors

## 故障排除 / Troubleshooting

### 模型无法加载 / Model Fails to Load

- 检查文件路径是否正确（区分大小写）/ Check file paths are correct (case-sensitive)
- 确保所有纹理文件存在 / Ensure all texture files exist
- 验证 JSON 文件格式正确 / Verify JSON file format is correct
- 查看浏览器控制台的错误信息 / Check browser console for error messages

### 模型显示异常 / Model Displays Incorrectly

- 确认模型版本为 Cubism 3.x 或 4.x / Confirm model version is Cubism 3.x or 4.x
- 检查纹理文件是否损坏 / Check if texture files are corrupted
- 验证 .moc3 文件完整性 / Verify .moc3 file integrity

## 许可证注意事项 / License Considerations

使用 Live2D 模型时，请注意：
When using Live2D models, please note:

1. **商业使用 / Commercial Use**: 某些模型可能不允许商业使用
   Some models may not allow commercial use
2. **署名要求 / Attribution**: 许多免费模型要求署名原作者
   Many free models require attribution to the original author
3. **再分发 / Redistribution**: 检查是否允许再分发模型文件
   Check if redistribution of model files is allowed
4. **修改 / Modification**: 确认是否允许修改模型
   Confirm if modification of the model is allowed

## 推荐的测试模型 / Recommended Test Models

对于开发和测试，推荐使用以下模型：
For development and testing, the following models are recommended:

1. **Haru** (来自 Live2D Cubism SDK / from Live2D Cubism SDK)
   - 简单的女性角色 / Simple female character
   - 包含基本动作和表情 / Includes basic motions and expressions

2. **Mark** (来自 Live2D Cubism SDK / from Live2D Cubism SDK)
   - 简单的男性角色 / Simple male character
   - 适合测试基本功能 / Good for testing basic functionality

## 技术规格 / Technical Specifications

- **支持的格式 / Supported Format**: .model3.json (Cubism 3.x/4.x)
- **纹理格式 / Texture Format**: PNG
- **推荐纹理大小 / Recommended Texture Size**: 2048x2048 或更小 / or smaller
- **动作格式 / Motion Format**: .motion3.json
- **表情格式 / Expression Format**: .exp3.json

## 参考资源 / Reference Resources

- Live2D 官方网站 / Official Website: https://www.live2d.com/
- Live2D 文档 / Documentation: https://docs.live2d.com/
- pixi-live2d-display 文档 / Documentation: https://guansss.github.io/pixi-live2d-display/
- Live2D 社区 / Community: https://community.live2d.com/

---

**最后更新 / Last Updated**: 2025-01-20
