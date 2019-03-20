# ruggine-protos-core

Contains the core ruggine gRPC Protobuf schemas.

## Protobuf / gRPC Directory structure
<pre>
|-protos
  |-messages
    |-foo.proto
    |-bar.proto
    |-...
  |-services
    |-foo.proto
    |-bar.proto
    |-...
</pre>

The directory structure is self explanatory:
- **messages** directory defines all of the message protobufs
  - each file in the directory is a separate package, i.e., maps to its own module
  - base package name: **oysterpack.ruggine.protos.core.messages**
- **services** directory defines all grpc services
  - each file in the directory is a separate package, i.e., maps to its own module
  - base package name: **oysterpack.ruggine.protos.core.services**
  
## src directory structure
<pre>
|-src
  |-messages
    |-foo.rs
    |-bar.rs
    |-...
  |-services
    |-foo.rs
    |-bar.rs
    |-...
  |-protos
    |-messages
      |-foo.rs
      |-bar.rs
      |-...
    |-services
      |-foo.rs
      |-bar.rs
      |-...
  |-lib.rs
  |-messages.rs
  |-protos.rs
  |-services.rs
</pre>

- **protos** module contains the Protobuf / gRPC generated code
- **messages** contains higher level application business logic code that is layered on top of low level protobuf messages
  - e.g., validation rules, constructors, conversions, etc
- **services** gRPC service implementations 

## How to share centrally managed gRPC Protobuf schemas
External git projects can import the schemas using the [git-subrepo](https://github.com/ingydotnet/git-subrepo) technique.

## Best Practices

### Version all packages
In order to support API evolution, version all packages by appending a version to the package name in the form of `v1`:
```proto
package oysterpack.ruggine.protos.core.messages.ulid.v1;
```

### Optimize for speed
via the `optimize_for` option:
```proto
syntax = "proto3";

option optimize_for = SPEED;

package oysterpack.ruggine.protos.core.messages.ulid.v1;
```
