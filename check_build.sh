#!/bin/bash
# 检查 GitHub Actions 构建状态

RUN_ID="28430599576"

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
