#include "common.h"

int main() {
    // Initialize
    int server_socket = 0, client_socket = 0;
    struct sockaddr_in serv_addr;
    char sendBuff[BUFFER+1];
    char recvBuff[BUFFER+1]; 

    // Zero out memory of serv_addr.
    memset(&serv_addr, '0', sizeof(serv_addr));

    // Initialize values of struct serv_addr.
    serv_addr.sin_family = AF_INET;    
    serv_addr.sin_addr.s_addr = htonl(INADDR_ANY); 
    serv_addr.sin_port = htons(PORT);  
    
    // Connect to a socket.
    server_socket = Socket(AF_INET, SOCK_STREAM, 0);  
    
    // Bind socket.
    Bind(server_socket, (struct sockaddr*)&serv_addr, sizeof(serv_addr));

    // Listen on server.
    Listen(server_socket, MAXPENDING);

    // Accept incoming request from client.
    client_socket = Accept(server_socket, (struct sockaddr*)NULL ,NULL);

    pid_t pid = fork();
    if (pid) {
        while (strncmp(recvBuff, ":exit", 5) != 0) {
            if(recv(client_socket, recvBuff, sizeof(recvBuff), 0) < 0)
                printf("Error: Receive\nErrno: %d\n", errno);
            recvBuff[BUFFER] = '\n';

            printf("Client: %s", recvBuff);
        }

        wait(&pid);
        close(client_socket);

    } else {
        while (strncmp(sendBuff, ":exit", 5) != 0) {
            fputs("(Server) Enter a message: ", stdout);
            fgets(sendBuff, sizeof(sendBuff), stdin);

            if(send(client_socket, sendBuff, sizeof(sendBuff), 0) < 0)
                printf("Error: Send\nErrno: %d\n", errno);
        }
        
        close(client_socket);
        exit(0);
    }

    close(server_socket);
}
