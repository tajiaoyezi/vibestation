# MVP-10 Sentry spike command log

## Environment checks

- Date: 2026-04-25
- SENTRY_DSN present: no
- SENTRY_AUTH_TOKEN present: no
- cargo-bloat present before spike: no
    Updating crates.io index
      Adding sentry v0.47.0 to dependencies
             Features:
             + backtrace
             + contexts
             + debug-images
             + httpdate
             + native-tls
             + panic
             + release-health
             + reqwest
             + sentry-backtrace
             + sentry-contexts
             + sentry-debug-images
             + sentry-panic
             + tokio
             + transport
             23 deactivated features
    Updating crates.io index
     Locking 81 packages to latest compatible versions
      Adding actix-codec v0.5.2
      Adding actix-http v3.12.1
      Adding actix-router v0.5.4
      Adding actix-rt v2.11.0
      Adding actix-server v2.6.0
      Adding actix-service v2.0.3
      Adding actix-utils v3.0.1
      Adding actix-web v4.13.0
      Adding addr2line v0.25.1
      Adding backtrace v0.3.76
      Adding base64ct v1.8.3
      Adding bytestring v1.5.0
      Adding cfg_aliases v0.2.1
      Adding convert_case v0.10.0
      Adding debugid v0.8.0
      Adding der v0.8.0
      Adding encoding_rs v0.8.35
      Adding findshlibs v0.10.2
      Adding foreign-types v0.3.2
      Adding foreign-types-shared v0.1.1
      Adding gimli v0.32.3
      Adding h2 v0.4.13
      Adding hostname v0.4.2
      Adding http v0.2.12
      Adding httpdate v1.0.3
      Adding hyper-rustls v0.27.9
      Adding hyper-tls v0.6.0
      Adding impl-more v0.1.9
      Adding language-tags v0.3.2
      Adding local-waker v0.1.4
      Adding native-tls v0.2.18
      Adding nix v0.30.1
      Adding objc2-cloud-kit v0.3.2
      Adding objc2-core-data v0.3.2
      Adding objc2-core-image v0.3.2
      Adding objc2-core-location v0.3.2
      Adding objc2-core-text v0.3.2
      Adding objc2-user-notifications v0.3.2
      Adding object v0.37.3
      Adding openssl v0.10.78
      Adding openssl-macros v0.1.1
      Adding openssl-probe v0.2.1
      Adding os_info v3.14.0
      Adding pem-rfc7468 v1.0.0
      Adding rand v0.9.4
      Adding rand_chacha v0.9.0
      Adding rand_core v0.9.5
      Adding regex-lite v0.1.9
      Adding ring v0.17.14
      Adding rustc-demangle v0.1.27
      Adding rustls v0.23.39
      Adding rustls-pki-types v1.14.1
      Adding rustls-webpki v0.103.13
      Adding schannel v0.1.29
      Adding security-framework v3.7.0
      Adding security-framework-sys v2.17.0
      Adding sentry v0.47.0
      Adding sentry-actix v0.47.0
      Adding sentry-backtrace v0.47.0
      Adding sentry-contexts v0.47.0
      Adding sentry-core v0.47.0
      Adding sentry-debug-images v0.47.0
      Adding sentry-panic v0.47.0
      Adding sentry-tracing v0.47.0
      Adding sentry-types v0.47.0
      Adding serde_urlencoded v0.7.1
      Adding signal-hook-registry v1.4.8
      Adding socket2 v0.5.10
      Adding subtle v2.6.1
      Adding tokio-native-tls v0.3.1
      Adding tokio-rustls v0.26.4
      Adding tracing-attributes v0.1.31
      Adding tracing-subscriber v0.3.23
      Adding uname v0.1.1
      Adding untrusted v0.9.0
      Adding ureq v3.3.0
      Adding ureq-proto v0.6.0
      Adding utf8-zero v0.8.1
      Adding valuable v0.1.1
      Adding webpki-root-certs v1.0.7
      Adding zeroize v1.8.2

## Step 5 · Spike dependency cleanup

```text
    Removing sentry from dependencies
```
