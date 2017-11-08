#include <stdint.h>
#include <stdlib.h>

// Opaque struct which mimics complex Swift/Rust structs.
struct Opaque;

// Create the Naamio client service.
struct Opaque *create_service(uint8_t threads);

// This can only be called once, and level can only be in 0..6
// (calling more than once, or invalid level has no effect)
void set_log_level(uint8_t level);

// Since it's allocated on the Rust side, it should be deallocated there.
void drop_service(struct Opaque *rust_service);

// Has effect only if it's a valid URI (default: "http://localhost:8000")
void set_naamio_host(const char *addr);

/* Plugin registration */

struct RegisterRequest {
    const char *name;
    const char *rel_url;
    const char *endpoint;
};

// NOTE: The closure represents a Swift class with a context-captured closure.

void register_plugin(void *closure,
                     struct Opaque *rust_service,
                     struct RegisterRequest *req,
                     void (*callback)(void *closure, const char *token));

void register_plugin_with_host(void *closure,
                               struct Opaque *rust_service,
                               const char *host,
                               struct RegisterRequest *req,
                               void (*callback)(void *closure));
