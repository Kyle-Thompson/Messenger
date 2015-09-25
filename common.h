#include <sys/socket.h>
#include <sys/types.h>
#include <sys/wait.h>
#include <netinet/in.h>
#include <arpa/inet.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>
#include <errno.h>
#include <string.h>
#include <netdb.h>

#define BUFFER 1024
#define PORT 5000
#define MAXPENDING 3
#define SERVER_IP "127.0.0.1"

int Socket(int, int, int);
void Bind(int, const struct sockaddr *, socklen_t);
void Listen(int, int);
int Accept(int, struct sockaddr *restrict, socklen_t *restrict);
void Connect(int, const struct sockaddr *, socklen_t);
int Send(int, const void *, size_t, int);
int Recv(int, void *, size_t, int);