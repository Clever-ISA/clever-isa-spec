# mdbook-fiction-template
Template repo using CI with mdbook and mdbook-fiction-tools

## CI

CI setup is provided under .github/workflows/mdbook.yml. The configuration will build both a github page and epub artifacts on push to the `dist` branch, or on workflow_dispatch. You can modify the file accordingly if you wish to use another branch for page builds (like `main`). 
You need to set up github pages on github actions (and configure the environment appropriately).

Third party and other git host workflows are not yet implemented (PRs welcome for standard CI systems).

## Required Tools

To run locally you need:
* cargo, which can be installed via rustup
* mdbook, which can be installed by `cargo install mdbook@^0.4.37` or <https://github.com/rust-lang/mdBook/releases>, with a version compatible with 0.4.37.
* mdbook-fiction-tools, which can be install by `cargo install --git https://github.com/chorman0773/mdbook-fiction-tools>`. 

Once tools are installed, you can run `mdbook build` or `mdbook serve` as usual for an mdbook and view it 
