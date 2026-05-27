#!/bin/bash
# Build the GNX Language Specification PDF from the Typst source.
# Stamps the current git short hash and build timestamp into the
# document's title page so a printed copy is traceable to a revision.

set -e

cd "$(dirname "$0")"

GIT_HASH=$(git rev-parse --short HEAD 2>/dev/null || echo "unknown")
BUILD_TIME=$(date '+@ %H:%M:%S UTC%:z')

echo "Building GNX Language Specification..."
echo "  Git Hash:   $GIT_HASH"
echo "  Build Time: $BUILD_TIME"

typst compile \
  --input githash="$GIT_HASH" \
  --input buildtime="$BUILD_TIME" \
  gnx_language_spec.typ

if [ $? -eq 0 ]; then
    echo "✓ PDF generated: gnx_language_spec.pdf"
    ls -lh gnx_language_spec.pdf
else
    echo "✗ Build failed"
    exit 1
fi
