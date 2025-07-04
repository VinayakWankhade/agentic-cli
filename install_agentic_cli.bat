@echo off
setlocal

REM Set source and destination paths
set "SRC=%~dp0target\release\agentic-cli.exe"
set "DEST=%USERPROFILE%\.cargo\bin\agentic-cli.exe"

REM Create destination directory if it doesn't exist
if not exist "%USERPROFILE%\.cargo\bin" mkdir "%USERPROFILE%\.cargo\bin"

REM Copy the executable (overwrite if exists)
copy /Y "%SRC%" "%DEST%"

if %ERRORLEVEL%==0 (
    echo agentic-cli.exe has been installed to %%USERPROFILE%%\.cargo\bin!
    echo You may need to add %%USERPROFILE%%\.cargo\bin to your PATH if you haven't already.
) else (
    echo Failed to copy agentic-cli.exe. Please make sure it was built successfully.
)

pause
endlocal 