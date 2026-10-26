"""Native Rust desktop packages. Local builds only; no Git or remote operations."""

from __future__ import annotations

import argparse
import json
import os
import platform
import plistlib
import re
import shutil
import subprocess
import sys
import tempfile
import tomllib
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[1]
CONFIG = json.loads((ROOT / "packaging/app.json").read_text())
PACKAGE = tomllib.loads((ROOT / "Cargo.toml").read_text())["package"]
NAME, BIN, VERSION = CONFIG["name"], CONFIG["binary"], PACKAGE["version"]
DIST, BUILD = ROOT / "dist", ROOT / "build"
TOOLS = BUILD / "package-tools"
TOOL_PYTHON = TOOLS / ("Scripts/python.exe" if os.name == "nt" else "bin/python")
CARGO = os.environ.get("CARGO", "cargo")


def run(*args: str | Path, **options: Any) -> subprocess.CompletedProcess[Any]:
    return subprocess.run([str(a) for a in args], cwd=ROOT, check=True, **options)


def host_target() -> str:
    output = run("rustc", "-vV", capture_output=True, text=True).stdout
    host = next(
        line.removeprefix("host: ")
        for line in output.splitlines()
        if line.startswith("host: ")
    )
    if sys.platform == "win32":
        return str(CONFIG["windows_target"])
    if (sys.platform == "darwin" and "apple-darwin" not in host) or (
        sys.platform.startswith("linux") and "linux-gnu" not in host
    ):
        raise ValueError("Use a native Rust toolchain for the current OS.")
    return str(host)


def inno() -> str | None:
    result = os.environ.get("ISCC") or shutil.which("ISCC") or shutil.which("ISCC.exe")
    if result:
        return result
    for variable in ("ProgramFiles(x86)", "ProgramFiles", "LOCALAPPDATA"):
        for suffix in ("Inno Setup 6/ISCC.exe", "Programs/Inno Setup 6/ISCC.exe"):
            candidate = Path(os.environ.get(variable, "")) / suffix
            if candidate.is_file():
                return str(candidate)
    return None


def inspector() -> str | None:
    for name in ("dumpbin.exe", "llvm-objdump.exe", "objdump.exe", "objdump"):
        result = shutil.which(name)
        if result:
            return result
    return None


def doctor() -> int:
    print(f"{NAME} {VERSION} | {platform.system()} / {platform.machine()}", flush=True)
    missing = [name for name in (CARGO, "rustc", "rustup") if not shutil.which(name)]
    if not missing:
        print(run("rustc", "--version", capture_output=True, text=True).stdout.strip())
        print("Target:", host_target())
    if sys.platform == "darwin":
        missing.extend(
            name
            for name in ("codesign", "hdiutil", "otool", "install_name_tool", "ditto")
            if not shutil.which(name)
        )
        if not TOOL_PYTHON.exists():
            missing.append("project DMG tools: run setup")
        else:
            result = subprocess.run(
                [str(TOOL_PYTHON), "-c", "import dmgbuild; from PIL import Image"],
                cwd=ROOT,
                capture_output=True,
                check=False,
            )
            if result.returncode:
                missing.append("DMG Python dependencies: run setup")
    elif sys.platform == "win32":
        if not inno():
            missing.append("Inno Setup 6.3+ (ISCC.exe)")
        if not inspector():
            missing.append("dumpbin.exe or objdump.exe for runtime DLL inspection")
        required = (
            ("gcc", "make", "m4", "windres")
            if "gnu" in CONFIG["windows_target"]
            else ("cl.exe", "rc.exe")
        )
        missing.extend(name for name in required if not shutil.which(name))
    elif sys.platform.startswith("linux"):
        missing.extend(
            name
            for name in (
                "pkg-config",
                "dpkg-deb",
                "dpkg-shlibdeps",
                "dpkg",
                "cc",
                "xdg-open",
                *CONFIG.get("extra_build_tools", []),
            )
            if not shutil.which(name)
        )
        for package in CONFIG.get("linux_pkgconfig", ["xkbcommon"]):
            if (
                shutil.which("pkg-config")
                and subprocess.run(
                    ["pkg-config", "--exists", package], check=False
                ).returncode
            ):
                missing.append(f"pkg-config dependency: {package}")
    else:
        missing.append("native macOS, Windows or Ubuntu host")
    for item in missing:
        print("MISSING:", item)
    print(
        "Read-only diagnostics. No system packages, services, permissions or credentials changed."
    )
    return int(bool(missing))


def setup() -> None:
    if sys.platform == "darwin":
        if BUILD.is_symlink():
            raise ValueError("build must not be a symlink")
        BUILD.mkdir(exist_ok=True)
        if not TOOL_PYTHON.exists():
            run(sys.executable, "-m", "venv", TOOLS)
        run(
            TOOL_PYTHON,
            "-m",
            "pip",
            "install",
            "-r",
            ROOT / "scripts/requirements-package.txt",
        )
    run("rustup", "target", "add", host_target())
    run("rustup", "component", "add", "rustfmt", "clippy")
    print(
        "Project tools prepared. Native compiler/system packages are diagnosed by doctor."
    )


def require_closed() -> None:
    if sys.platform == "darwin":
        processes = run(
            "/bin/ps", "-axo", "comm=", capture_output=True, text=True
        ).stdout.splitlines()
        if any(line.strip().startswith(str(DIST) + "/") for line in processes):
            raise ValueError(
                f"Close the generated {NAME} application before rebuilding. No process was stopped."
            )
    elif sys.platform == "win32":
        run(
            "powershell.exe",
            "-NoProfile",
            "-Command",
            f"if (Get-Process -Name '{BIN}' -ErrorAction SilentlyContinue) {{ Write-Error 'Close {NAME} before rebuilding'; exit 1 }}",
        )
    else:
        for entry in Path("/proc").glob("[0-9]*/exe"):
            try:
                if entry.resolve().is_relative_to(DIST):
                    raise ValueError(
                        f"Close {NAME} before rebuilding. No process was stopped."
                    )
            except (OSError, RuntimeError):
                continue


def compile_release(target: str) -> tuple[dict[str, Path], set[str]]:
    environment = dict(os.environ)
    if sys.platform == "darwin":
        environment.setdefault("MACOSX_DEPLOYMENT_TARGET", "11.0")
    args = [
        CARGO,
        "build",
        "--release",
        "--locked",
        "--target",
        target,
        "--message-format=json-render-diagnostics",
    ]
    binaries = [BIN, *CONFIG.get("cli_binaries", [])]
    for binary in binaries:
        args.extend(["--bin", binary])
    result = run(*args, env=environment, capture_output=True, text=True)
    if result.stderr:
        print(result.stderr, file=sys.stderr, end="")
    artifacts: dict[str, Path] = {}
    packages: set[str] = set()
    for line in result.stdout.splitlines():
        try:
            message = json.loads(line)
        except ValueError:
            continue
        if message.get("reason") == "compiler-message":
            print(
                message.get("message", {}).get("rendered", ""), file=sys.stderr, end=""
            )
        if message.get("reason") == "compiler-artifact":
            packages.add(message["package_id"])
            if message.get("executable") and message["target"]["name"] in binaries:
                artifacts[message["target"]["name"]] = Path(message["executable"])
    if set(artifacts) != set(binaries):
        raise ValueError(
            "Cargo did not report all requested executables. Previous packages were preserved."
        )
    return artifacts, packages


def public_resources(destination: Path, packages: set[str]) -> None:
    destination.mkdir(parents=True, exist_ok=True)
    for source in [
        *ROOT.glob("README*.md"),
        ROOT / "LICENSE",
        *(ROOT / name for name in CONFIG.get("resources", [])),
    ]:
        resource = destination / source.relative_to(ROOT)
        resource.parent.mkdir(parents=True, exist_ok=True)
        if source.is_dir():
            shutil.copytree(
                source,
                resource,
                ignore=shutil.ignore_patterns(".DS_Store", "Icon", "Icon\r"),
            )
        elif source.is_file():
            shutil.copy2(source, resource)
    notices = destination / "third-party-licenses"
    notices.mkdir()
    cargo_home = Path(os.environ.get("CARGO_HOME", Path.home() / ".cargo"))
    for package in sorted(packages):
        match = re.search(r"#([^#@]+)@([^@]+)$", package)
        if not match or not package.startswith("registry+"):
            continue
        crate = "-".join(match.groups())
        for folder in (cargo_home / "registry/src").glob(f"*/{crate}"):
            for source in folder.rglob("*"):
                if source.is_file() and any(
                    source.name.upper().startswith(word)
                    for word in ("LICENSE", "COPYING", "NOTICE")
                ):
                    output = notices / crate / source.relative_to(folder)
                    output.parent.mkdir(parents=True, exist_ok=True)
                    shutil.copyfile(source, output)
    (notices / "cargo-packages.json").write_text(
        json.dumps(sorted(packages), indent=2) + "\n"
    )


def mac_libraries(app: Path, binaries: list[Path]) -> None:
    frameworks = app / "Contents/Frameworks"
    frameworks.mkdir()
    pending = list(binaries)
    seen: set[Path] = set()
    while pending:
        binary = pending.pop()
        if binary in seen:
            continue
        seen.add(binary)
        dependencies = run(
            "otool", "-L", binary, capture_output=True, text=True
        ).stdout.splitlines()[1:]
        for line in dependencies:
            dependency = line.strip().split(" (", 1)[0]
            if dependency.startswith(
                (
                    "/System/Library/",
                    "/usr/lib/",
                    "@rpath/",
                    "@loader_path/",
                    "@executable_path/",
                )
            ):
                if (
                    dependency.startswith("@")
                    and not (frameworks / Path(dependency).name).exists()
                ):
                    raise ValueError(
                        f"Unresolved application library: {dependency}; use a self-contained release toolchain."
                    )
                continue
            source = Path(dependency)
            if not source.is_file():
                raise ValueError(f"Missing linked library: {source}")
            output = frameworks / source.name
            if not output.exists():
                shutil.copyfile(source, output)
                output.chmod(0o755)
                run("install_name_tool", "-id", "@rpath/" + source.name, output)
                pending.append(output)
            run(
                "install_name_tool",
                "-change",
                dependency,
                "@rpath/" + source.name,
                binary,
            )
    for binary in binaries:
        if any(frameworks.iterdir()):
            run(
                "install_name_tool",
                "-add_rpath",
                "@executable_path/../Frameworks",
                binary,
            )


def mac_package(
    stage: Path, artifacts: dict[str, Path], packages: set[str], label: str
) -> tuple[Path, list[Path]]:
    from dmgbuild import build_dmg
    from PIL import Image, ImageDraw, ImageFont

    label = "arm64" if label == "aarch64" else label
    app = stage / f"{NAME}.app"
    executables = app / "Contents/MacOS"
    resources = app / "Contents/Resources"
    executables.mkdir(parents=True)
    for name, source in artifacts.items():
        shutil.copy2(source, executables / name)
    public_resources(resources, packages)
    plist = plistlib.loads((ROOT / "packaging/macos/Info.plist").read_bytes())
    plist.update(CFBundleShortVersionString=VERSION, CFBundleVersion=VERSION)
    (app / "Contents/Info.plist").write_bytes(plistlib.dumps(plist))
    shutil.copyfile(ROOT / CONFIG["mac_icon"], resources / plist["CFBundleIconFile"])
    mac_libraries(app, [executables / name for name in artifacts])
    identity = os.environ.get(CONFIG["signing_variable"], "-")
    run("codesign", "--force", "--deep", "--sign", identity, app)
    run("codesign", "--verify", "--deep", "--strict", app)
    background = stage / "background.png"
    image = Image.new("RGB", (760, 480), "#f5f0e3")
    draw = ImageDraw.Draw(image)
    draw.text((42, 36), NAME, font=ImageFont.load_default(size=42), fill="#173e34")
    draw.text(
        (44, 94),
        f"Version {VERSION} | Drag the app to Applications",
        font=ImageFont.load_default(size=19),
        fill="#285d49",
    )
    draw.line((325, 265, 435, 265), fill="#285d49", width=4)
    draw.line((419, 251, 435, 265, 419, 279), fill="#285d49", width=4)
    image.save(background)
    dmg = stage / f"{NAME}-{VERSION}-macos-{label}.dmg"

    def verify_copy(mount_point: str, _options: Any) -> None:
        run("codesign", "--verify", "--deep", "--strict", Path(mount_point) / app.name)

    build_dmg(
        str(dmg),
        NAME,
        settings={
            "files": [str(app)],
            "symlinks": {"Applications": "/Applications"},
            "icon": str(ROOT / CONFIG["mac_icon"]),
            "background": str(background),
            "window_rect": ((180, 180), (760, 480)),
            "icon_locations": {app.name: (208, 268), "Applications": (552, 268)},
            "icon_size": 100,
            "text_size": 14,
            "default_view": "icon-view",
            "format": "UDZO",
            "show_toolbar": False,
            "show_status_bar": False,
            "show_sidebar": False,
            "include_icon_view_settings": True,
            "create_hook": verify_copy,
        },
    )
    run("hdiutil", "verify", dmg)
    archive = stage / f"{NAME}-macos-{label}.zip"
    run("ditto", "-c", "-k", "--norsrc", "--keepParent", app, archive)
    return app, [dmg, archive]


def windows_libraries(bundle: Path) -> None:
    tool = inspector()
    if not tool:
        raise ValueError(
            "A DLL dependency inspector is required; run from Developer PowerShell/MSYS2."
        )
    system = Path(os.environ["SystemRoot"]) / "System32"
    search = [
        bundle,
        *(Path(part) for part in os.environ.get("PATH", "").split(os.pathsep)),
        system,
    ]
    pending = list(bundle.glob("*.exe"))
    seen: set[str] = set()
    while pending:
        binary = pending.pop()
        if binary.name.lower() in seen:
            continue
        seen.add(binary.name.lower())
        flag = "/DEPENDENTS" if "dumpbin" in Path(tool).name.lower() else "-p"
        output = run(tool, flag, binary, capture_output=True, text=True).stdout
        dependencies = re.findall(
            r"(?im)^\s*(?:DLL Name:\s*)?([\w.+-]+\.dll)\s*$", output
        )
        if not dependencies:
            raise ValueError(
                f"No DLL import table reported for {binary.name}; verify the inspector."
            )
        for dependency in dependencies:
            lower = dependency.lower()
            if lower.startswith(("api-ms-win-", "ext-ms-win-")):
                continue
            # VC runtime redistributables must travel with the package if present.
            if (system / dependency).exists() and not lower.startswith(
                ("vcruntime", "msvcp", "libgcc", "libstdc", "libwinpthread")
            ):
                continue
            source = next(
                (
                    folder / dependency
                    for folder in search
                    if (folder / dependency).is_file()
                ),
                None,
            )
            if source is None:
                raise ValueError(
                    f"Missing runtime DLL: {dependency}. No incomplete installer was promoted."
                )
            target = bundle / dependency
            if not target.exists():
                shutil.copy2(source, target)
                pending.append(target)


def windows_package(
    stage: Path, artifacts: dict[str, Path], packages: set[str], label: str
) -> tuple[Path, list[Path]]:
    bundle = stage / CONFIG["windows_directory"]
    bundle.mkdir()
    for name, source in artifacts.items():
        filename = CONFIG["windows_executable"] if name == BIN else name + ".exe"
        shutil.copy2(source, bundle / filename)
    public_resources(bundle, packages)
    windows_libraries(bundle)
    compiler = inno()
    if not compiler:
        raise ValueError("Inno Setup 6.3+ is required")
    installer_name = f"{NAME}-{VERSION}-windows-{label}-setup"
    run(
        compiler,
        f"/DAppName={NAME}",
        f"/DAppVersion={VERSION}",
        f"/DAppId={CONFIG['identifier']}",
        f"/DExeName={CONFIG['windows_executable']}",
        f"/DAppArch={'arm64' if label == 'aarch64' else 'x64compatible'}",
        f"/DProjectDir={ROOT}",
        f"/DBundleDir={bundle}",
        f"/DOutputDir={stage}",
        f"/DOutputName={installer_name}",
        f"/DIconFile={ROOT / CONFIG['windows_icon']}",
        ROOT / "packaging/installer.iss",
    )
    archive = Path(
        shutil.make_archive(str(stage / f"{NAME}-windows-x64"), "zip", bundle)
    )
    return bundle, [stage / (installer_name + ".exe"), archive]


def linux_package(
    stage: Path, artifacts: dict[str, Path], packages: set[str], label: str
) -> tuple[Path, list[Path]]:
    slug = CONFIG["linux_name"]
    bundle = stage / slug
    bundle.mkdir()
    for name, source in artifacts.items():
        shutil.copy2(source, bundle / name)
    public_resources(bundle, packages)
    deb = stage / "deb"
    prefix = deb / "opt" / slug
    shutil.copytree(bundle, prefix)
    for name in artifacts:
        launcher = deb / "usr/bin" / (slug if name == BIN else name)
        launcher.parent.mkdir(parents=True, exist_ok=True)
        launcher.write_text(f'#!/bin/sh\nexec "/opt/{slug}/{name}" "$@"\n')
        launcher.chmod(0o755)
    desktop = deb / f"usr/share/applications/{slug}.desktop"
    desktop.parent.mkdir(parents=True)
    desktop.write_text(
        f"[Desktop Entry]\nType=Application\nName={NAME}\nExec=/usr/bin/{slug}\nIcon={slug}\nTerminal=false\nCategories=Utility;\n"
    )
    icon = deb / f"usr/share/icons/hicolor/128x128/apps/{slug}.png"
    icon.parent.mkdir(parents=True)
    shutil.copyfile(ROOT / CONFIG["linux_icon"], icon)
    control = stage / "debian/control"
    control.parent.mkdir()
    control.write_text(
        f"Source: {slug}\nSection: utils\nPriority: optional\nMaintainer: d9911\n\nPackage: {slug}\nArchitecture: any\nDescription: {NAME} desktop application\n"
    )
    command = ["dpkg-shlibdeps", "-O", *(f"-e{bundle / name}" for name in artifacts)]
    result = subprocess.run(
        command, cwd=stage, check=True, capture_output=True, text=True
    )
    dependencies = next(
        (
            line.removeprefix("shlibs:Depends=")
            for line in result.stdout.splitlines()
            if line.startswith("shlibs:Depends=")
        ),
        "",
    )
    dependencies = ", ".join(filter(None, [dependencies, *CONFIG["linux_runtime"]]))
    architecture = run(
        "dpkg", "--print-architecture", capture_output=True, text=True
    ).stdout.strip()
    if {"aarch64": "arm64", "x86_64": "amd64"}.get(label) != architecture:
        raise ValueError("Target architecture differs from the native Debian host")
    info = deb / "DEBIAN/control"
    info.parent.mkdir()
    info.write_text(
        f"Package: {slug}\nVersion: {VERSION}\nArchitecture: {architecture}\nMaintainer: d9911\nSection: utils\nPriority: optional\nDepends: {dependencies}\nDescription: {NAME} desktop application\n"
    )
    guard = f"""#!/bin/sh
set -eu
for process in /proc/[0-9]*/cmdline; do
    [ -r "$process" ] || continue
    command=$(tr '\\000' '\\n' < "$process" 2>/dev/null | head -n 1)
    case "$command" in
        /opt/{slug}/*|/usr/bin/{slug}) echo "Close {NAME} before updating or removing it." >&2; exit 1 ;;
    esac
done
exit 0
"""
    for name in ("preinst", "prerm"):
        script = deb / "DEBIAN" / name
        script.write_text(guard)
        script.chmod(0o755)
    notice = deb / f"usr/share/doc/{slug}/copyright"
    notice.parent.mkdir(parents=True)
    shutil.copyfile(ROOT / "LICENSE", notice)
    installer = stage / f"{slug}_{VERSION}_{architecture}.deb"
    # Container/CI umasks may be 000; Debian control directories must not be world-writable.
    for entry in deb.rglob("*"):
        if entry.is_dir():
            entry.chmod(0o755)
        elif entry.is_file():
            entry.chmod(0o755 if entry.stat().st_mode & 0o111 else 0o644)
    run("dpkg-deb", "--root-owner-group", "--build", deb, installer)
    run("dpkg-deb", "--info", installer)
    return bundle, [installer]


def promote(
    stage: Path, application: Path, installers: list[Path], target: str
) -> None:
    manifest = stage / "package.json"
    manifest.write_text(
        json.dumps(
            {
                "name": NAME,
                "version": VERSION,
                "target": target,
                "host": platform.platform(),
                "application": application.name,
                "installers": [path.name for path in installers],
            },
            indent=2,
        )
        + "\n"
    )
    outputs = [application, *installers, manifest]
    # Validate every destination before replacing even the first output.
    for source in outputs:
        if (DIST / source.name).is_symlink():
            raise ValueError(
                "Refusing to replace an output symlink; previous packages preserved"
            )
    require_closed()
    backup = stage / "previous-release"
    backup.mkdir()
    saved: list[tuple[Path, Path]] = []
    moved: list[Path] = []
    try:
        for source in outputs:
            destination = DIST / source.name
            if destination.exists():
                previous = backup / source.name
                destination.rename(previous)
                saved.append((previous, destination))
            source.rename(destination)
            moved.append(destination)
    except BaseException:
        for destination in reversed(moved):
            destination.rename(stage / ("incomplete-" + destination.name))
        for previous, destination in reversed(saved):
            previous.rename(destination)
        raise


def build() -> Path:
    if doctor():
        raise ValueError("Resolve doctor diagnostics before building.")
    if BUILD.is_symlink() or DIST.is_symlink():
        raise ValueError("build/dist must be ordinary project directories")
    require_closed()
    BUILD.mkdir(exist_ok=True)
    DIST.mkdir(exist_ok=True)
    target = host_target()
    label = target.split("-", 1)[0]
    with (BUILD / "package.lock").open("a+b") as lock:
        if os.name == "nt":
            msvcrt: Any = __import__("msvcrt")
            lock.write(b"0")
            lock.flush()
            lock.seek(0)
            msvcrt.locking(lock.fileno(), msvcrt.LK_NBLCK, 1)
        else:
            import fcntl

            fcntl.flock(lock.fileno(), fcntl.LOCK_EX | fcntl.LOCK_NB)
        artifacts, packages = compile_release(target)
        with tempfile.TemporaryDirectory(prefix="package-", dir=BUILD) as directory:
            stage = Path(directory)
            packager = (
                mac_package
                if sys.platform == "darwin"
                else windows_package if sys.platform == "win32" else linux_package
            )
            application, installers = packager(stage, artifacts, packages, label)
            if not installers or not all(path.is_file() for path in installers):
                raise ValueError("Installer missing; previous output preserved")
            promote(stage, application, installers, target)
            final = DIST / application.name
            print("Application:", final)
            for installer in installers:
                print("Package:", DIST / installer.name)
            return DIST / installers[0].name


def open_path(path: Path) -> None:
    if sys.platform == "darwin":
        run("open", path)
    elif sys.platform == "win32":
        os.startfile(str(path))
    else:
        run("xdg-open", path)


def run_release() -> None:
    manifest = json.loads((DIST / "package.json").read_text())
    path = DIST / manifest["application"]
    if sys.platform == "darwin":
        open_path(path)
    else:
        executable = path / (CONFIG["windows_executable"] if os.name == "nt" else BIN)
        if not executable.is_file():
            raise ValueError("Release missing; run build first")
        subprocess.Popen(
            [str(executable)], cwd=executable.parent, start_new_session=True
        )


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "action",
        nargs="?",
        default="help",
        choices=[
            "help",
            "doctor",
            "setup",
            "dev",
            "build",
            "local",
            "run",
            "start",
            "cli",
        ],
    )
    parser.add_argument("--platform", choices=["macos", "windows", "linux"])
    args = parser.parse_args()
    actual = (
        "macos"
        if sys.platform == "darwin"
        else "windows" if sys.platform == "win32" else "linux"
    )
    if args.platform and args.platform != actual:
        raise ValueError(
            f"Build {args.platform} packages on their native host; current host: {actual}"
        )
    if args.action == "help":
        print(
            f"{NAME}: make <action> / ./package.ps1 <action>\nVersion from Cargo.toml: {VERSION}\n"
        )
        for action, meaning in (
            ("doctor", "Read-only native tool diagnosis"),
            ("setup", "Prepare project packaging tools"),
            ("dev", "Existing Cargo development GUI"),
            ("build", "Native release + installer + compatible archive"),
            ("local", "Build and open installer for manual installation"),
            ("run", "Open existing release without rebuilding"),
            ("start", "Build and run release"),
            ("cli", "Existing headless CLI, where provided"),
        ):
            print(f"  {action:8} {meaning}")
        print("Output:", DIST)
        return 0
    if args.action == "setup":
        setup()
        return 0
    if args.action == "doctor":
        return doctor()
    if args.action in ("dev", "cli"):
        binary = (
            BIN
            if args.action == "dev"
            else next(iter(CONFIG.get("cli_binaries", [])), None)
        )
        if not binary:
            raise ValueError("This application has no separate CLI")
        return subprocess.call([CARGO, "run", "--locked", "--bin", binary], cwd=ROOT)
    if args.action in ("build", "local", "start"):
        if sys.platform == "darwin" and Path(sys.prefix) != TOOLS:
            if not TOOL_PYTHON.is_file():
                raise ValueError("Run setup first to prepare project-only DMG tools")
            return subprocess.call(
                [str(TOOL_PYTHON), str(Path(__file__).resolve()), args.action], cwd=ROOT
            )
        installer = build()
        if args.action == "local":
            open_path(installer)
        elif args.action == "start":
            run_release()
        return 0
    run_release()
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except subprocess.CalledProcessError as error:
        if error.stderr:
            print(error.stderr, file=sys.stderr)
        if error.stdout:
            for line in error.stdout.splitlines():
                try:
                    message = json.loads(line)
                    if message.get("reason") == "compiler-message":
                        print(message["message"].get("rendered", ""), file=sys.stderr)
                except ValueError:
                    print(line, file=sys.stderr)
        print(f"ERROR: {error}", file=sys.stderr)
        raise SystemExit(1)
    except (OSError, ValueError) as error:
        print(f"ERROR: {error}", file=sys.stderr)
        raise SystemExit(1)
