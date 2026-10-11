# anyknown

anyknown is a web browser written in [Aye](https://github.com/aye-lang/aye), for people and AI agents at once; its sources are in `browser/` (see `browser/README.md` and `docs/browser.md`), and this repository builds it with the Aye compiler pinned in `AYE_REV`. Install that compiler with `cargo install --git https://github.com/aye-lang/aye --rev $(cat AYE_REV) --locked aye` (or, with a checkout of Aye, set `AYE=/path/to/aye/target/release/aye`), install SDL2, FreeType and libcurl (HarfBuzz and brotli optional; on Ubuntu 24.04 run `sh scripts/setup-ubuntu.sh`), then run the commands below from the repository root. CI installs the compiler from the private `aye-lang/aye` repository, so the repository secret `AYE_TOKEN` must hold a token that can read it.

```
aye check browser
aye test browser
aye build browser                                   # writes ./anyknown
AYE=$(command -v aye) cargo test --release --test browser
```
