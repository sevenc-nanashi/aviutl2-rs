# Scripts

This directory contains automation scripts for the project.

## update_old_releases.rb

Automatically updates old GitHub releases to add a link to the latest release.

### Usage

```bash
GITHUB_TOKEN=your_token ruby scripts/update_old_releases.rb
```

### Dry Run Mode

To test without making actual changes:

```bash
GITHUB_TOKEN=your_token DRY_RUN=true ruby scripts/update_old_releases.rb
```

### Automation

This script is automatically run by the `Update Old Releases` GitHub Actions workflow whenever a new release is published.

## test_update_logic.rb

Unit test for the update_old_releases.rb logic. Can be run without API credentials.

```bash
ruby scripts/test_update_logic.rb
```
