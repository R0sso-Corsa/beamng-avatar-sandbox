"""Package only mod/ into an installable ZIP; run from any directory."""
from pathlib import Path
from zipfile import ZIP_DEFLATED, ZipFile

ROOT = Path(__file__).resolve().parents[1]


def main():
    files = sorted(p for p in (ROOT / "mod").rglob("*") if p.is_file())
    if not files:
        raise SystemExit("No mod files found")
    output = ROOT / "dist" / "beamng-avatar-sandbox.zip"
    output.parent.mkdir(exist_ok=True)
    with ZipFile(output, "w", ZIP_DEFLATED) as archive:
        for path in files:
            archive.write(path, path.relative_to(ROOT / "mod").as_posix())
    print(output)


if __name__ == "__main__":
    main()
