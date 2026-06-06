# Beacon

Beacon is a dynamic DNS tool for use with AWS & Route53.

It lets you update a single AWS hosted zone to point at your current IP ad infinitum, e.g. for running a home server.

## Usage

```
beacon
    --zone-name <name>      name of the hosted zone to update, e.g. your-domain.com
    --update-root           if set, will update the root record matching your zone
    --subdomain <name>      name of subdomain(s) to update. use repeatedly to add multiple
    --interval <integer>    if set, repeats every n seconds. otherwise, exits after operation
```

## Authentication

Beacon expects to be run in an environment where AWS credentials are already configured - this could be using the AWS CLI SSO, a machine with IAM roles, or access keys. See the [AWS documentation](https://docs.aws.amazon.com/sdkref/latest/guide/access.html) for your options in that regard.

## Installation

Currently only installation from the source code is supported. This means a Rust installation is required.

```
cargo install --git https://github.com/chrislewisdev/beacon
```

## Examples

To update a basic domain name once:

```bash
beacon --zone-name staticlinkage.dev --update-root
```

To update multiple subdomains every 5 minutes:

```bash
beacon --zone-name staticlinkage.dev --subdomain wiki --subdomain portfolio --interval 300
```
