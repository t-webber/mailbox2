package main

//#include <stdlib.h>
import "C"

import (
	"context"
	"fmt"
	"os"
	"path/filepath"
	"sync"

	"github.com/adrg/xdg"
	_ "github.com/mattn/go-sqlite3"
	"go.mau.fi/whatsmeow"
	"go.mau.fi/whatsmeow/store"
	"go.mau.fi/whatsmeow/store/sqlstore"
	"go.mau.fi/whatsmeow/types/events"
	waLog "go.mau.fi/whatsmeow/util/log"
)

var (
	client        *whatsmeow.Client
	clientMu      sync.Mutex
	needs_pairing bool
)

//export wa_needs_pairing
func wa_needs_pairing() bool {
	return needs_pairing
}

//export wa_init_client
func wa_init_client() *C.char {
	if client != nil {
		return nil
	}

	db_path := xdg.DataHome + "/.mailbox/wa.db"

	if err := os.MkdirAll(filepath.Dir(db_path), 0755); err != nil {
		return C.CString(fmt.Sprintf("create db error:%s", err.Error()))
	}

	db_params := "_foreign_keys=on&_journal_mode=WAL&_busy_timeout=5000"
	container, err := sqlstore.New(context.Background(), "sqlite3", "file:"+db_path+"?"+db_params, nil)
	if err != nil {
		return C.CString(fmt.Sprintf("get container error:%s", err.Error()))
	}

	devices, err := container.GetAllDevices(context.Background())
	if err != nil {
		return C.CString(fmt.Sprintf("get_all_devices_error:%s", err.Error()))
	}

	var deviceStore *store.Device
	if len(devices) > 0 {
		deviceStore = devices[0]
		needs_pairing = false
	} else {
		deviceStore = container.NewDevice()
		needs_pairing = true
	}

	client = whatsmeow.NewClient(deviceStore, waLog.Stdout("Client", "DEBUG", true))
	client.AddEventHandler(logPairingStatus)
	return nil
}

func logPairingStatus(evt interface{}) {
	switch v := evt.(type) {
	case *events.PairSuccess:
		fmt.Println("Paired! JID:", v.ID)
	case *events.Connected:
		fmt.Println("Connected and logged in")
	case *events.LoggedOut:
		fmt.Println("Logged out:", v.Reason)
	}
}

//export wa_pair_phone
func wa_pair_phone(phone *C.char) *C.char {
	clientMu.Lock()
	defer clientMu.Unlock()

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

	needs_pairing = false
	return C.CString(code)
}

func main() {}
