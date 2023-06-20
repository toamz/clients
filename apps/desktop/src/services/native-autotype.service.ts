import { ipcMain, app, globalShortcut } from "electron";

import { LogService } from "@bitwarden/common/platform/abstractions/log.service";
import { autotypes } from "@bitwarden/desktop-native";

import { WindowMain } from "../main/window.main";

export class NativeAutoTypeService {
  constructor(private mainWindow: WindowMain, private logService: LogService) {
    ipcMain.on("autotype", async (event: any, username: string, pass: string) => {
      this.autotype(username, pass);
    });

    app.whenReady().then(() => {
      const ret = globalShortcut.register("CommandOrControl+Alt+V", () => {
        autotypes.activeWindowTitle().then((title) => {
          mainWindow.win.webContents.send("autotype-sortcut", title);
        });
      });

      if (!ret) {
        logService.error("Unable to register auto-type shortcut");
      }
    });

    app.on("will-quit", () => {
      globalShortcut.unregister("CommandOrControl+Alt+V");
    });
  }

  async autotype(username: string, pass: string) {
    autotypes.sendLogin(username, pass);
  }
}
