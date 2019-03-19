# ruggine-protos-core

Contains the core ruggine gRPC Protobuf schemas.

## Directory structure
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
- the **messages** directory defines all of the message protobufs
  - each file in the directory is a separate package, i.e., maps to its own module
  - base package name: **oysterpack.ruggine.protos.core.messages**
- the **services** directory defines all grpc services
  - each file in the directory is a separate package, i.e., maps to its own module
  - base package name: **oysterpack.ruggine.protos.core.services**

## How to share centrally managed gRPC Protobuf schemas
External git projects can import the schemas using the [git-subrepo](https://github.com/ingydotnet/git-subrepo) technique.
