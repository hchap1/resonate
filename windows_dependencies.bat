@echo off
echo Installing Dependencies...
winget install winget install --id Google.Chrome --exact --silent --accept-package-agreements --accept-source-agreements
echo CHROME installed [1/3]
winget install Chromium.ChromeDriver
echo CHROMEDRIVER installed [2/3]
winget install SQLite.SQlite
echo SQLITE installed [3/3]
echo FINISHED
pause
