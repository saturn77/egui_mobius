#!/bin/bash
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Run checks first
./scripts/check_release.sh

VERSION=$(grep '^version' crates/egui_mobius/Cargo.toml | head -n1 | cut -d'"' -f2)
echo -e "${YELLOW}Publishing version $VERSION${NC}"

# Order of crates matters due to dependencies
CRATES=(
    "as_command_derive"
    "egui_mobius_macros"
    "egui_mobius_widgets"
    "egui_mobius"
)

# Publish each crate
for crate in "${CRATES[@]}"; do
    echo -e "${YELLOW}Publishing $crate...${NC}"
    (cd "crates/$crate" && cargo publish --allow-dirty)
    # Wait between publishes to ensure crates.io index is updated
    sleep 30
done

echo -e "${GREEN}✓ All crates published!${NC}"

# Update badges in README
echo -e "${YELLOW}Updating README badges...${NC}"
sed -i 's/crates.io-unreleased-orange/crates.io-v'"$VERSION"'-blue/' README.md

# Commit badge updates
git add README.md
git commit -m "docs: update badges for $VERSION release"
git push

echo -e "${GREEN}✓ Release $VERSION complete!${NC}"
echo -e "\nNext steps:"
echo "1. Create GitHub release: https://github.com/saturn77/egui_mobius/releases/new"
echo "2. Update version numbers for next development cycle"
echo "3. Announce release in GitHub Discussions"
