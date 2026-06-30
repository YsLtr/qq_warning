#!/bin/bash
# 检查 GitHub Actions 构建状态

RUN_ID="28432326126"

echo "=== QQ Warning v0.1.0 构建状态 ==="
echo ""

# 获取构建状态
gh run view $RUN_ID

echo ""
echo "---"
echo ""
echo "💡 提示:"
echo "  - 查看实时日志: gh run view $RUN_ID --log"
echo "  - 在浏览器中查看: gh run view $RUN_ID --web"
echo "  - 等待完成: gh run watch $RUN_ID"
echo ""
echo "📝 说明:"
echo "  - 使用 rustls 替代 OpenSSL (纯 Rust TLS 实现,无系统依赖)"
echo "  - 使用 cross 工具构建 ARM64 版本"
echo "  - 已配置 contents:write 权限用于创建 Release"
