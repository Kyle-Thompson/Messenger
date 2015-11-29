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

int Accept(int socket, struct sockaddr *address, socklen_t *address_len) {
    int file_descriptor = 0;
    if ((file_descriptor = accept(socket, address, address_len)) < 0) {
        printf("Error: accept()\nErrno: %d %s\n", errno, strerror(errno));
        exit(1);
    }
    return file_descriptor;
}

void Connect(int socket, const struct sockaddr *address, socklen_t address_len) {
    if (connect(socket, address, address_len) < 0) {
        printf("Error: connect()\nErrno: %d %s\n", errno, strerror(errno));
        exit(1);
    }
}

int Send(int sockfd, const void *buf, size_t len, int flags) {
    int msg_length = 0;
    if ((msg_length = send(sockfd, buf, len, flags)) < 0) {
        printf("Error: send()\nErrno: %d %s\n", errno, strerror(errno));
        exit(1);
    }
    return msg_length;
}

int Recv(int sockfd, void *buf, size_t len, int flags) {
    int msg_length = 0;
    if ((msg_length = recv(sockfd, buf, len, flags)) < 0) {
        printf("Error: recv()\nErrno: %d %s\n", errno, strerror(errno));
        exit(1);
    }
    return msg_length;
}
