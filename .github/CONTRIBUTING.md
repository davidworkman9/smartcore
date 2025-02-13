# **Contributing**

When contributing to this repository, please first discuss the change you wish to make via issue,
email, or any other method with the owners of this repository before making a change.

Please note we have a [code of conduct](CODE_OF_CONDUCT.md), please follow it in all your interactions with the project.

## Pull Request Process

1. Open a PR following the template.
2. Update the CHANGELOG.md with details of changes to the interface if they are breaking changes, this includes new environment variables, exposed ports useful file locations and container parameters.
3. Pull Request can be merged in once you have the sign-off of one other developer, or if you do not have permission to do that you may request the reviewer to merge it for you.

## Issue Report Process

1. Go to the project's issues.
2. Select the template that better fits your issue.
3. Read carefully the instructions and write within the template guidelines.
4. Submit it and wait for support.

## Reviewing process

1. After a PR is opened maintainers are notified
2. Probably changes will be required to comply with the workflow, these commands are run automatically and all tests shall pass:
    * **Coverage** (optional): `tarpaulin` is used with command `cargo tarpaulin --out Lcov --all-features -- --test-threads 1`
    * **Linting**: `clippy` is used with command `cargo clippy --all-features -- -Drust-2018-idioms -Dwarnings`
    * **Testing**: multiple test pipelines are run for different targets
3. When everything is OK, code is merged.


## Contribution Best Practices

* Read this [how-to about Github workflow here](https://guides.github.com/introduction/flow/) if you are not familiar with.

* Read all the texts related to [contributing for an OS community](https://github.com/HTTP-APIs/hydrus/tree/master/.github).

* Read this [how-to about writing a PR](https://github.com/blog/1943-how-to-write-the-perfect-pull-request) and this [other how-to about writing a issue](https://wiredcraft.com/blog/how-we-write-our-github-issues/)
  
* **read history**: search past open or closed issues for your problem before opening a new issue.

* **PRs on develop**: any change should be PRed first in `development`

* **testing**:  everything should work and be tested as defined in the workflow. If any is failing for non-related reasons, annotate the test failure in the PR comment.
