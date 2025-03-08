@echo off
echo Installing Dependencies...
winget install winget install --id Google.Chrome --exact --silent --accept-package-agreements --accept-source-agreements
echo CHROME installed [1/4]
winget install Chromium.ChromeDriver
echo CHROMEDRIVER installed [2/4]
winget install SQLite.SQlite
echo SQLITE installed [3/4]
winget install yt-dlp
echo YTDLP installed [4/4]
echo FINISHED
pause
