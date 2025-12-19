# TOON-LD Specification

This directory contains the formal specification for TOON-LD written in [Bikeshed](https://github.com/tabatkins/bikeshed).

## Viewing the Specification

The rendered specification will be available at: **https://kushbisen.github.io/toon-ld/**

(After you enable GitHub Pages and push changes)

## Building Locally

### Prerequisites

Bikeshed requires Python 3.12 or higher. Install using pipx:

```bash
# Install pipx if you don't have it
brew install pipx

# Install Bikeshed
pipx install bikeshed

# Update Bikeshed data files
bikeshed update
```

Or with pip in a virtual environment:

```bash
python3.12 -m venv venv
source venv/bin/activate
pip install bikeshed
bikeshed update
```

### Build the Spec

```bash
cd spec
bikeshed spec index.bs index.html
```

This will generate `index.html` from the `index.bs` source file.

### Watch Mode

For development, you can use watch mode to automatically rebuild on changes:

```bash
bikeshed watch index.bs index.html
```

Then open `index.html` in your browser and it will auto-refresh on changes.

## Deployment

The specification is automatically built and deployed to GitHub Pages whenever changes are pushed to the `main` branch. See `.github/workflows/deploy-spec.yml` for the deployment configuration.

### Enable GitHub Pages

1. Go to your repository settings
2. Navigate to **Pages** section
3. Under "Build and deployment", set **Source** to "GitHub Actions"
4. Push changes to `main` branch
5. The workflow will automatically build and deploy the spec

## File Structure

- `index.bs` - Bikeshed source file (edit this)
- `index.html` - Generated HTML specification (auto-generated, do not edit manually)
- `README.md` - This file

## What's Included

The specification now includes:

- **Complete grammar** - Full EBNF grammar for TOON-LD
- **All data types** - Null, boolean, numbers, strings, objects, arrays
- **Tabular arrays** - The key feature for token reduction
- **JSON-LD support** - All JSON-LD 1.1 keywords and features
- **Value nodes** - Compact notation for language tags and datatypes
- **Parsing & serialization algorithms** - Step-by-step implementation guidance
- **Conformance levels** - Three levels of implementation conformance
- **Examples** - Multiple real-world examples with JSON-LD comparison
- **Security considerations** - Input validation, URI handling, resource limits
- **IANA considerations** - Media type and file extension recommendations
- **Appendices** - Comparison with JSON-LD, token savings analysis, changelog

## Contributing

To make changes to the specification:

1. Edit `index.bs`
2. Build locally to preview changes: `bikeshed spec index.bs index.html`
3. Review the generated `index.html` in your browser
4. Commit and push to `main` branch to trigger automatic deployment

### Bikeshed Markup Tips

- Use `{#anchor-id}` after headings to set custom IDs
- Use `<div class='example'>` for code examples
- Use `[[#section-id]]` for internal cross-references
- Use `[[!RFC2119]]` for external spec references
- Nested lists need 4-space indentation for sub-items
- Code blocks use triple backticks with language tags

## Resources

- [Bikeshed Documentation](https://speced.github.io/bikeshed/)
- [Bikeshed Metadata Reference](https://tabatkins.github.io/bikeshed/#metadata)
- [Example Specifications](https://github.com/tabatkins/bikeshed/tree/main/tests)
- [W3C Specifications](https://www.w3.org/TR/) - Examples of real specs using Bikeshed