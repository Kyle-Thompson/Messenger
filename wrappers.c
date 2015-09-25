#include "common.h"

int Socket(int domain, int type, int protocol) {
    int socket_number = 0;
    if ((socket_number = socket(domain, type, protocol)) < 0) {
        printf("Error: socket()\nErrno: %d %s\n", errno, strerror(errno));
        exit(1);
    }
    return socket_number;
}

void Bind(int sockfd, const struct sockaddr *addr, socklen_t addrlen) {
    if (bind(sockfd, addr, addrlen) < 0) {
        printf("Error: bind()\nErrno: %d %s\n", errno, strerror(errno));
        exit(1);
    }
}

void Listen(int sockfd, int backlog) {
    if (listen(sockfd, backlog) < 0) {
        printf("Error: listen()\nErrno: %d %s\n", errno, strerror(errno));
        exit(1);
    }
}

int Accept(int socket, struct sockaddr *restrict address, socklen_t *restrict address_len) {
    int file_descriptor = 0;
    if ((file_descriptor = accept(socket, address, address_len)) < 0) {
        printf("Error: accept()\nErrno: %d %s\n", errno, strerror(errno));
        exit(1);
    }
    return file_descriptor;
}