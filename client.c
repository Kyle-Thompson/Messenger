#include "common.h"

int main(void) {
    int sockfd = 0;
    char recvBuff[BUFFER+1];
    char sendBuff[BUFFER+1];
    struct sockaddr_in serv_addr;

    memset(recvBuff, '0' ,sizeof(recvBuff));

    sockfd = Socket(AF_INET, SOCK_STREAM, 0);

    serv_addr.sin_family = AF_INET;
    serv_addr.sin_port = htons(PORT);
    serv_addr.sin_addr.s_addr = inet_addr("127.0.0.1");
    
    Connect(sockfd, (struct sockaddr *)&serv_addr, sizeof(serv_addr));

    pid_t pid = fork();
    if (pid) {
        while (strncmp(recvBuff, ":exit", 5) != 0) {
            Recv(sockfd, recvBuff, sizeof(recvBuff), 0);
            recvBuff[BUFFER] = '\n';

            printf("Server: %s", recvBuff);
        }

        wait(&pid);
        close(sockfd);

    } else {
        while (strncmp(sendBuff, ":exit", 5) != 0) {
            fputs("(Client) Enter a message: ", stdout);
            fgets(sendBuff, sizeof(sendBuff), stdin);

            Send(sockfd, sendBuff, sizeof(sendBuff), 0);
        }
        close(sockfd);
        exit(0);
    }
}