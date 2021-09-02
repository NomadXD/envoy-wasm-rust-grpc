package main

import (
	"context"
	"fmt"
	"log"
	"net"

	api "github.com/NomadXD/envoy-wasm-rust-grpc/service/proto"
	"github.com/google/uuid"
	"google.golang.org/grpc"
	"google.golang.org/grpc/reflection"
)

type server struct {
	api.UnimplementedExampleServiceServer
}

func (*server) GenerateHeader(ctx context.Context, req *api.Request) (*api.Response, error) {
	fmt.Printf("GenerateHeader invoked for request with %v:%v", req.PathType, req.Id)
	if req.GetPathType() == "REQUEST_PATH" {
		id := "REQ" + uuid.NewString()
		res := &api.Response{
			PathType: "REQUEST_PATH",
			Header:   id,
		}
		return res, nil
	} else if req.GetPathType() == "RESPONSE_PATH" {
		id := "RES" + uuid.NewString()
		res := &api.Response{
			PathType: "RESPONSE_PATH",
			Header:   id,
		}
		return res, nil
	} else {
		return nil, fmt.Errorf("Error handling request with %v:%v", req.PathType, req.Id)
	}
}

func main() {
	fmt.Println("Starting gRPC service .....")
	lis, err := net.Listen("tcp", "0.0.0.0:50051")
	if err != nil {
		log.Fatalf("Error starting gRPC service: %v", err)
	}
	service := grpc.NewServer()
	api.RegisterExampleServiceServer(service, &server{})
	reflection.Register(service)
	if err := service.Serve(lis); err != nil {
		log.Fatalf("Failed to serve: %v", err)
	}
	fmt.Println("gRPC service listening on port 50051")
}
