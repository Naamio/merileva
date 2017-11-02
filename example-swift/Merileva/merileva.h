#include <stdint.h>
#include <stdlib.h>

struct naamio_service;

struct naamio_service *create_service(uint8_t threads);
void drop_service(struct naamio_service *ptr);
