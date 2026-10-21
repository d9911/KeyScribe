#!/bin/sh
set -eu
project_dir=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
cd "$project_dir"
export MACOSX_DEPLOYMENT_TARGET=11.0
rustup target add aarch64-apple-darwin
cargo build --release --locked --target aarch64-apple-darwin
mkdir -p dist/Keyscribe.app/Contents/MacOS dist/Keyscribe.app/Contents/Resources
cp target/aarch64-apple-darwin/release/keyscribe dist/Keyscribe.app/Contents/MacOS/keyscribe.new
mv -f dist/Keyscribe.app/Contents/MacOS/keyscribe.new dist/Keyscribe.app/Contents/MacOS/keyscribe
cp packaging/macos/Info.plist dist/Keyscribe.app/Contents/Info.plist
cp assets/icons/keyscribe.icns dist/Keyscribe.app/Contents/Resources/Keyscribe.icns
for readme in README.md README.ru.md README.es.md LICENSE; do
    cp "$readme" "dist/Keyscribe.app/Contents/Resources/$readme"
done
# Mirror source resources, removing obsolete files and excluding Finder metadata.
finder_icon_cr=$(printf 'Icon\r')
rsync -a --delete --delete-excluded --exclude='.DS_Store' --exclude='Icon' --exclude="$finder_icon_cr" assets/ dist/Keyscribe.app/Contents/Resources/assets/
rsync -a --delete --delete-excluded --exclude='.DS_Store' --exclude='Icon' --exclude="$finder_icon_cr" samples/ dist/Keyscribe.app/Contents/Resources/samples/
# Reuse a real signing identity when supplied; ad hoc builds can need fresh TCC consent.
codesign --force --sign "${KEYSCRIBE_SIGNING_IDENTITY:--}" dist/Keyscribe.app
ditto -c -k --norsrc --keepParent dist/Keyscribe.app dist/Keyscribe-macos-arm64.zip
echo "Готово: $project_dir/dist/Keyscribe.app"
