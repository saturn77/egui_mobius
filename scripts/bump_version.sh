#!/bin/bash
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

if [ "$#" -ne 1 ]; then
    echo "Usage: $0 NEW_VERSION"
    echo "Example: $0 0.3.0-alpha.2"
    exit 1
fi

NEW_VERSION=$1
echo -e "${YELLOW}Bumping version to $NEW_VERSION${NC}"

# Update all crate versions
for crate in crates/*/Cargo.toml; do
    echo -e "${YELLOW}Updating $crate...${NC}"
    sed -i 's/^version.*=.*$/version = "'"$NEW_VERSION"'"/' "$crate"
done

# Update README badge
sed -i 's/version-[0-9]\+\.[0-9]\+\.[0-9]\+\(-[a-z0-9.]\+\)\?-green/version-'"$NEW_VERSION"'-green/' README.md

# Add new section to CHANGELOG.md
DATE=$(date +%Y-%m-%d)
sed -i "4i\\\n## [$NEW_VERSION] - $DATE\n\n### Added\n\n### Changed\n\n### Fixed\n" CHANGELOG.md

echo -e "${GREEN}✓ Version bumped to $NEW_VERSION${NC}"
echo -e "\nNext steps:"
echo "1. Review changes: git diff"
echo "2. Update CHANGELOG.md with planned changes"
echo "3. Commit changes: git commit -am 'chore: bump version to $NEW_VERSION'"
