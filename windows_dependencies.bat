@echo off
echo Installing Dependencies...
winget install winget install --id Google.Chrome --exact --silent --accept-package-agreements --accept-source-agreements
echo CHROME installed [1/5]
winget install Chromium.ChromeDriver
echo CHROMEDRIVER installed [2/5]
winget install SQLite.SQlite
echo SQLITE installed [3/4]
winget install yt-dlp
echo YTDLP installed [4/5]
winget install Gyan.FFmpeg
echo FFMPEG installed [5/5]
echo FINISHED
pause
