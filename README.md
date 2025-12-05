# mspy-fix
解决微软拼音在中文状态下斜杠是顿号的问题。

##### 编译
`cargo build --release`

这是已经编译好的可以直接使用的可执行文件（ [mspy.exe](mspy-fix.exe) ）。

##### 运行
双击即可，此后控制台窗口会一直存在，按 `Ctrl + C` 终止运行。

如果想隐藏控制台窗口，使用以下 PowerShell 命令运行 `mspy-fix` ：

`Start-Process -WindowStyle Hidden -FilePath "mspy-fix.exe"`
