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

## Examples

To update a basic domain name once:

```bash
beacon --zone-name staticlinkage.dev --update-root
```

To update multiple subdomains every 5 minutes:

```bash
beacon --zone-name staticlinkage.dev --subdomain wiki --subdomain portfolio --interval 300
```

## Authentication

Beacon expects to be run in an environment where AWS credentials are already configured - this could be using the AWS CLI SSO, a machine with IAM roles, or access keys. See the [AWS documentation](https://docs.aws.amazon.com/sdkref/latest/guide/access.html) for your options in that regard.

### Required IAM Permissions

The following permissions are required to successfully update your DNS:

```json
{
    "Version": "2012-10-17",
    "Statement": [
        {
            "Sid": "VisualEditor0",
            "Effect": "Allow",
            "Action": "route53:ChangeResourceRecordSets",
            "Resource": "arn:aws:route53:::hostedzone/<hosted-zone-id>"
        },
        {
            "Sid": "VisualEditor1",
            "Effect": "Allow",
            "Action": "route53:ListHostedZones",
            "Resource": "*"
        }
    ]
}
```

Note that `hosted-zone-id` is NOT the same as your domain - it will be a string of letters and numbers displayed against your Hosted Zones in the AWS Console.

## Installation

Currently you must build the project from source. On a computer with Rust installed, you can use `cargo install`:

```
cargo install --git https://github.com/chrislewisdev/beacon
```

If you need to build the project for another architecture e.g. a home Raspberry Pi server, I had the smoothest experience using [zigbuild](https://github.com/rust-cross/cargo-zigbuild). The following example builds for a Raspberry Pi 2, using GCC version 2.31 explicitly since that's the highest version my Pi had:

```
cargo zigbuild --release --target armv7-unknown-linux-gnueabihf.2.31
```
