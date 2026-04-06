# Live2D 模型快速开始指南 / Quick Start Guide

## 快速开始 / Quick Start

### 方法 1: 自动下载（推荐）/ Method 1: Auto-Download (Recommended)

```bash
cd FairyField/public/models/default
bash download-sample-model.sh
```

按照提示选择一个示例模型（推荐 Haru 或 Hiyori）。
Follow the prompts to select a sample model (Haru or Hiyori recommended).

### 方法 2: 手动下载 / Method 2: Manual Download

1. 访问 / Visit: https://www.live2d.com/en/download/cubism-sdk/
2. 下载 "Cubism SDK for Web"
3. 解压并找到 `Samples/Resources/` 目录
4. 复制任一模型文件夹（如 Haru）的所有内容到 `public/models/default/`

### 验证安装 / Verify Installation

```bash
cd FairyField/public/models/default
node verify-model.cjs
```

如果看到绿色的 "✓ 验证通过"，说明模型已正确安装。
If you see green "✓ Verification PASSED", the model is correctly installed.

## 目录结构 / Directory Structure

```
public/models/
├── default/              # 默认模型目录
│   ├── README.md        # 详细文档
│   ├── model3.json      # 模型配置（需要下载）
│   ├── *.moc3           # 模型数据（需要下载）
│   ├── textures/        # 纹理文件（需要下载）
│   └── ...
└── QUICK_START.md       # 本文件
```

## 常见问题 / FAQ

### Q: 为什么没有预装模型？/ Why is there no pre-installed model?

A: Live2D 官方示例模型受许可证保护，不能直接分发。我们提供了自动下载脚本来帮助您获取。
Live2D official sample models are protected by license and cannot be directly distributed. We provide an auto-download script to help you obtain them.

### Q: 可以使用自己的模型吗？/ Can I use my own model?

A: 可以！只需将您的模型文件放到 `public/models/default/` 目录，确保有 `model3.json` 文件即可。
Yes! Just place your model files in `public/models/default/` directory and ensure there's a `model3.json` file.

### Q: 模型加载失败怎么办？/ What if the model fails to load?

A: 运行验证脚本检查问题：
Run the verification script to check for issues:

```bash
node verify-model.cjs
```

脚本会告诉您缺少哪些文件。
The script will tell you which files are missing.

### Q: 支持哪些模型格式？/ What model formats are supported?

A: 目前支持 Live2D Cubism 3.x 和 4.x 的 .model3.json 格式。
Currently supports Live2D Cubism 3.x and 4.x .model3.json format.

## 更多信息 / More Information

详细文档请查看：
For detailed documentation, see:

- `public/models/default/README.md` - 完整使用指南
- `public/models/default/model3.json.example` - 配置示例

## 许可证 / License

使用 Live2D 模型时请遵守相应的许可证条款。
Please comply with the respective license terms when using Live2D models.

官方示例模型许可证：
Official sample model license:
https://www.live2d.com/en/terms/live2d-free-material-license-agreement/

---

**需要帮助？/ Need Help?**

查看详细文档：`public/models/default/README.md`
See detailed documentation: `public/models/default/README.md`
