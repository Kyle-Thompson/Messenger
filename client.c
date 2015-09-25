#include "common.h"

int main(void) {
    // Initialize socket_fd and the buffers.
    int socket_fd = 0;
    char recvBuff[BUFFER+1], sendBuff[BUFFER+1];

    // Create and initialize serv_addr.
    struct sockaddr_in serv_addr;
    memset(recvBuff, '0', sizeof(recvBuff));
    serv_addr.sin_family = AF_INET;
    serv_addr.sin_port = htons(PORT);
    serv_addr.sin_addr.s_addr = inet_addr(SERVER_IP);
    
    // Connect to a socket.
    socket_fd = Socket(AF_INET, SOCK_STREAM, 0);

    // Connect to other client.
    Connect(socket_fd, (struct sockaddr *)&serv_addr, sizeof(serv_addr));

    pid_t pid = fork();
    if (pid) { // Handle incoming messages.
        while (strncmp(recvBuff, ":exit", 5) != 0) {
            Recv(socket_fd, recvBuff, sizeof(recvBuff), 0);
            recvBuff[BUFFER] = '\n';

            printf("Server: %s", recvBuff);
        }

        wait(&pid);
        close(socket_fd);

    } else { // Handle outgoing messages.
        while (strncmp(sendBuff, ":exit", 5) != 0) {
            fputs("(Client) Enter a message: ", stdout);
            fgets(sendBuff, sizeof(sendBuff), stdin);

            Send(socket_fd, sendBuff, sizeof(sendBuff), 0);
        }

        close(socket_fd);
        exit(0);
    }
}