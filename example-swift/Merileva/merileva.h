#include <stdint.h>
#include <stdlib.h>

// Create an opaque struct which mimics the Rust struct.
struct naamio_service;

// Create the Naamio client service.
struct naamio_service *create_service(uint8_t threads);

// Since it's allocated on the Rust side, it should be deallocated there.
void drop_service(struct naamio_service *ptr);
