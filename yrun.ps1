#!/bin/bash
#!/bin/bash
echo .
echo .
echo .
echo .
echo .
echo .
echo .
echo .
echo . BUILD SEP
echo .
echo .
echo .
echo .
echo .
echo .
echo .
echo .
echo .
if($args[0] -ne "") {
  $EXAMPLE = $args[0]
} else {
  $EXAMPLE="button"
}

$TYPE="debug"
#$EXAMPLE="bitbang"

$PORT="COM6"
$TERM_BAUDRATE="19200"
$FLASH_BAUDRATE="115200"

# change this to the directory of where you built rustc for xtensa
$CUSTOM_RUSTC='C:\dev\rust-esp32-msvc\rust-xtensa'
$env:RUST_BACKTRACE='0'
$env:XARGO_RUST_SRC="$CUSTOM_RUSTC\src"
$env:RUSTC="$CUSTOM_RUSTC/build/x86_64-pc-windows-msvc/stage2/bin/rustc"
echo $env:RUSTC


cargo xbuild --example $EXAMPLE
if($LASTEXITCODE -ne 0)
{
  echo "compile failed."
  exit 0
}

# change this for release flashes
$BIN_PATH="target/xtensa-esp32-none-elf/$TYPE/examples/$EXAMPLE"

#display section seizes
#Write-Output ""
#xtensa-esp32-elf-readelf $BIN_PATH -S|egrep 'BIT|\[Nr\]' |awk 'BEGIN {FS="[ \t\[\]]+"}  $9~/A|Flg/ {size=sprintf("%7d", "0x" $7)+0; printf("%-3s %-20s %-8s %-8s %-8s %8s %-3s %-3s\n",$2,$3,$4,$5,$7,((size>0)?size:$7),$9,$12); total+=size; } END { printf("\nTotal: %d bytes\n",total)}'
#Write-Output ""

# convert to bin
if (Test-Path "$BIN_PATH.bin")
{
  Remove-Item "$BIN_PATH.bin"
}
Write-Host -NoNewline "Converting to bin file for flashing using: "
esptool.exe --chip esp32 elf2image --flash_mode="dio" --flash_freq "40m" --flash_size "4MB" -o "$BIN_PATH.bin" "$BIN_PATH"
if($LASTEXITCODE -ne 0)
{
  echo "bin file generation failed"
  exit 0;
}

# kill terminal programs using the same port
#ps -ef|grep $PORT|egrep -v "$0|grep" |awk '{print $2}'|xargs -r kill

# flash
Write-Host -NoNewline "Flashing via: "

esptool.exe --chip esp32 --port $PORT --baud $FLASH_BAUDRATE --before default_reset --after hard_reset write_flash -z --flash_mode dio --flash_freq 40m --flash_size detect 0x1000 "$BIN_PATH.bin"
if($LASTEXITCODE -ne 0)
{
  echo "flashing failed."
  exit 0;
}

# start terminal program
Write-Host ""
Write-Host "Starting terminal"
python3.exe -m serial.tools.miniterm $PORT $TERM_BAUDRATE


