#include "common.h"

int main() {
    int server_socket = 0, client_socket = 0;

    struct sockaddr_in serv_addr;

    char sendBuff[BUFFER+1];
    char recvBuff[BUFFER+1]; 

    memset(&serv_addr, '0', sizeof(serv_addr));
    memset(sendBuff, '0', sizeof(sendBuff));

    serv_addr.sin_family = AF_INET;    
    serv_addr.sin_addr.s_addr = htonl(INADDR_ANY); 
    serv_addr.sin_port = htons(PORT);  
    
    server_socket = socket(AF_INET, SOCK_STREAM, 0);  

    bind(server_socket, (struct sockaddr*)&serv_addr, sizeof(serv_addr));

    if (listen(server_socket, 10) == -1) {
        printf("Failed to listen\n");
        return -1;
    }

    client_socket = accept(server_socket, (struct sockaddr*)NULL ,NULL);

    pid_t pid = fork();
    if (pid) {
        while (strncmp(recvBuff, "exit", 4) != 0) {
            if(recv(client_socket, recvBuff, sizeof(recvBuff), 0) < 0)
                printf("Error: Receive\nErrno: %d\n", errno);
            recvBuff[BUFFER] = '\n';

            printf("Client: %s", recvBuff);
        }

        wait(&pid);
        close(client_socket);

    } else {
        while (strncmp(sendBuff, "exit", 4) != 0) {
            fputs("(Server) Enter a message: ", stdout);
            fgets(sendBuff, sizeof(sendBuff), stdin);

            if(send(client_socket, sendBuff, sizeof(sendBuff), 0) < 0)
                printf("Error: Send\nErrno: %d\n", errno);
        }
        
        close(client_socket);
        exit(0);
    }

    close(server_socket); 

    return 0;    

}
