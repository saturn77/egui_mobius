#!/bin/bash
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${YELLOW}Running release preparation checks...${NC}"

# Check we're on main branch
CURRENT_BRANCH=$(git branch --show-current)
if [ "$CURRENT_BRANCH" != "main" ]; then
    echo -e "${RED}❌ Not on main branch${NC}"
    exit 1
fi
echo -e "${GREEN}✓ On main branch${NC}"

# Check for uncommitted changes
if ! git diff --quiet HEAD; then
    echo -e "${RED}❌ Uncommitted changes present${NC}"
    exit 1
fi
echo -e "${GREEN}✓ No uncommitted changes${NC}"

# Check version consistency
VERSION=$(grep '^version' crates/egui_mobius/Cargo.toml | head -n1 | cut -d'"' -f2)
echo -e "${YELLOW}Checking version consistency for $VERSION...${NC}"

# Check all crate versions match
for crate in crates/*/Cargo.toml; do
    CRATE_VERSION=$(grep '^version' "$crate" | head -n1 | cut -d'"' -f2)
    if [ "$CRATE_VERSION" != "$VERSION" ]; then
        echo -e "${RED}❌ Version mismatch in $crate: $CRATE_VERSION${NC}"
        exit 1
    fi
done
echo -e "${GREEN}✓ All crate versions match${NC}"

# Check version in README badge
if ! grep -q "version-$VERSION-green.svg" README.md; then
    echo -e "${RED}❌ README badge version doesn't match${NC}"
    exit 1
fi
echo -e "${GREEN}✓ README badge version matches${NC}"

# Check CHANGELOG entry exists
if ! grep -q "## \[$VERSION\]" CHANGELOG.md; then
    echo -e "${RED}❌ No CHANGELOG entry for $VERSION${NC}"
    exit 1
fi
echo -e "${GREEN}✓ CHANGELOG entry exists${NC}"

# Run tests
echo -e "${YELLOW}Running tests...${NC}"
cargo test --all
echo -e "${GREEN}✓ All tests passed${NC}"

# Check examples build
echo -e "${YELLOW}Building examples...${NC}"
for example in examples/*; do
    if [ -f "$example/Cargo.toml" ]; then
        echo "Building $(basename $example)..."
        cargo build -p "$(basename $example)"
    fi
done
echo -e "${GREEN}✓ All examples built successfully${NC}"

# Run clippy
echo -e "${YELLOW}Running clippy...${NC}"
cargo clippy --all -- -D warnings
echo -e "${GREEN}✓ Clippy checks passed${NC}"

# Check documentation
echo -e "${YELLOW}Building documentation...${NC}"
cargo doc --no-deps
echo -e "${GREEN}✓ Documentation built successfully${NC}"

echo -e "${GREEN}✓ All release checks passed!${NC}"
echo -e "\nNext steps:"
echo "1. Tag release: git tag -a v$VERSION -m 'Release $VERSION'"
echo "2. Push tag: git push origin v$VERSION"
echo "3. Run: ./scripts/publish_release.sh"
