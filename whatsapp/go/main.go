package main

//#include <stdlib.h>
import "C"

import (
	"context"
	"fmt"
	"sync"

	"github.com/adrg/xdg"
	_ "github.com/mattn/go-sqlite3"
	"go.mau.fi/whatsmeow"
	"go.mau.fi/whatsmeow/store/sqlstore"
	waLog "go.mau.fi/whatsmeow/util/log"
)

var (
	client   *whatsmeow.Client
	clientMu sync.Mutex
)

func initClient() error {
	db_path := xdg.DataHome + ".mailbox" + "wa.db"
	container, err := sqlstore.New(context.Background(), "sqlite3", "file:"+db_path+"?_foreign_keys=on", nil)
	if err != nil {
		return err
	}

	deviceStore, err := container.GetFirstDevice(context.Background())
	if err != nil {
		return err
	}

	client = whatsmeow.NewClient(deviceStore, waLog.Stdout("Client", "DEBUG", true))
	return nil
}

//export wa_pair_phone
func wa_pair_phone(phone *C.char) *C.char {
	clientMu.Lock()
	defer clientMu.Unlock()

	if client == nil {
		if err := initClient(); err != nil {
			return C.CString(fmt.Sprintf("init_error:%s", err.Error()))
		}
	}

	if !client.IsConnected() {
		if err := client.Connect(); err != nil {
			return C.CString(fmt.Sprintf("connect_error:%s", err.Error()))
		}
	}

	code, err := client.PairPhone(
		context.Background(),
		C.GoString(phone),
		false,
		whatsmeow.PairClientChrome,
		"Chrome (Linux)",
	)
	if err != nil {
		return C.CString(fmt.Sprintf("pair_error:%s", err.Error()))
	}
	return C.CString(code)
}

func main() {}
