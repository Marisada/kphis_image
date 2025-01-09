@echo off

set pwa_path=volume\pwa
set project_path=%cd%

cd %pwa_path%

For /f "tokens=2-4 delims=/ " %%a in ('date /t') do (set date_now=%%c%%a%%b)
For /f "tokens=1-3 delims=/:." %%a in ("%TIME%") do (set time_now=%%a%%b%%c)

echo const VERSION = '%date_now%-%time_now%' > sw.js
type sw_template.js >> sw.js

echo update sw.js version to %date_now%-%time_now%
cd %project_path%

echo done.