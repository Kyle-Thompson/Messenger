#include "common.h"

int main() {
    // Initialize sockets and the buffers.
    int server_socket = 0, client_socket = 0;
    char recvBuff[BUFFER+1], sendBuff[BUFFER+1];

    // Create and initialize serv_addr.
    struct sockaddr_in serv_addr;
    memset(&serv_addr, '0', sizeof(serv_addr));
    serv_addr.sin_family = AF_INET;   
    serv_addr.sin_port = htons(PORT);  
    serv_addr.sin_addr.s_addr = htonl(INADDR_ANY); 
    
    // Connect to a socket.
    server_socket = Socket(AF_INET, SOCK_STREAM, 0);  
    
    // Bind socket.
    Bind(server_socket, (struct sockaddr*)&serv_addr, sizeof(serv_addr));

    // Listen.
    Listen(server_socket, MAXPENDING);

    // Accept incoming request from client.
    client_socket = Accept(server_socket, (struct sockaddr*)NULL ,NULL);

    pid_t pid = fork();
    if (pid) { // Handle incoming messages.
        while (strncmp(recvBuff, ":exit", 5) != 0) {
            Recv(client_socket, recvBuff, sizeof(recvBuff), 0);
            recvBuff[BUFFER] = '\n';

            printf("Client: %s", recvBuff);
        }

        wait(&pid);
        close(client_socket);

    } else { // Handle outgoing messages.
        while (strncmp(sendBuff, ":exit", 5) != 0) {
            fputs("(Server) Enter a message: ", stdout);
            fgets(sendBuff, sizeof(sendBuff), stdin);

            Send(client_socket, sendBuff, sizeof(sendBuff), 0);
        }
        
        close(client_socket);
        exit(0);
    }

    close(server_socket);
}