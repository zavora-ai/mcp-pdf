# Changelog

## [3.1.1] - 2026-08-25

### Security

- Upgraded all PDF parser and writer dependencies to `lopdf` 0.42 or newer,
  closing the deeply nested object stack-overflow advisory.
- Upgraded `pdf-extract` to 0.12 and `printpdf` to the maintained 0.12 API.
