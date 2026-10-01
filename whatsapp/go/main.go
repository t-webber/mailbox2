package main

//#include <stdlib.h>
//#include <stdint.h>
//#include <stdbool.h>

import (
	"C"
	"context"
	"fmt"
	"os"
	"path/filepath"
	"strconv"
	"strings"
	"sync"
	"unsafe"

	"github.com/adrg/xdg"
	_ "github.com/mattn/go-sqlite3"
	"github.com/rs/zerolog"
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
	syncDone      bool
	syncMu        sync.Mutex
)

func log(parts ...string) {
	fmt.Printf("\x1b[38;2;37;211;102mwhatsapp: %s\x1b[0m\n", strings.Join(parts, " "))
}

func cerr(size *C.int, msg string, err error) *C.char {
	log(msg, err.Error())
	return cstr(size, msg)
}

func cstr(size *C.int, msg string) *C.char {
	*size = C.int(len(msg))
	return (*C.char)(unsafe.Pointer(unsafe.StringData(msg)))
}

var DATA_DIR = xdg.DataHome + "/.mailbox/"

//export wa_needs_pairing
func wa_needs_pairing() bool {
	return needs_pairing
}

//export wa_is_synced
func wa_is_synced() bool {
	syncMu.Lock()
	defer syncMu.Unlock()
	return syncDone
}

//export wa_init_client
func wa_init_client(size *C.int) *C.char {
	if client != nil {
		return nil
	}

	db_path := DATA_DIR + "wa.db"

	if err := os.MkdirAll(filepath.Dir(db_path), 0755); err != nil {
		return cerr(size, "create db error:", err)
	}

	db_params := "_foreign_keys=on&_journal_mode=WAL&_busy_timeout=5000"
	container, err := sqlstore.New(context.Background(), "sqlite3", "file:"+db_path+"?"+db_params, nil)
	if err != nil {
		return cerr(size, "get container error:", err)
	}

	devices, err := container.GetAllDevices(context.Background())
	if err != nil {
		return cerr(size, "get all devices error:", err)
	}

	var deviceStore *store.Device
	if len(devices) > 0 {
		deviceStore = devices[0]
		needs_pairing = false
	} else {
		deviceStore = container.NewDevice()
		needs_pairing = true
	}

	logFile, err := os.OpenFile(DATA_DIR+"wa.log", os.O_CREATE|os.O_WRONLY|os.O_APPEND, 0644)
	if err != nil {
		return cerr(size, "open log file error:", err)
	}
	client = whatsmeow.NewClient(deviceStore, waLog.Zerolog(zerolog.New(logFile).With().Timestamp().Logger()))
	client.AddEventHandler(logPairingStatus)
	if !needs_pairing {
		if err := client.Connect(); err != nil {
			return cerr(size, "connect error:", err)
		}
	}
	return nil
}

func logPairingStatus(evt any) {
	switch v := evt.(type) {
	case *events.PairSuccess:
		log("Paired! JID:", v.ID.String())
	case *events.Connected:
		log("Connected and logged in")
	case *events.LoggedOut:
		log("Logged out:", v.Reason.String())
	case *events.OfflineSyncCompleted:
		syncMu.Lock()
		syncDone = true
		syncMu.Unlock()
		log("Offline sync completed:", strconv.Itoa(v.Count), "events")
	case *events.HistorySync:
		log("History sync received")
	}
}

//export wa_pair_phone
func wa_pair_phone(phone *C.char, size *C.int, success *C.bool) *C.char {
	*success = C.bool(false)
	clientMu.Lock()
	defer clientMu.Unlock()

	if !client.IsConnected() {
		if err := client.Connect(); err != nil {
			return cerr(size, "connect error:", err)
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
		return cerr(size, "pair_error:", err)
	}

	needs_pairing = false
	*success = C.bool(true)
	return cstr(size, code)
}

func main() {}
