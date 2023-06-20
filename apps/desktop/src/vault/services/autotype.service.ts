import { Injectable } from "@angular/core";
import { ipcRenderer } from "electron";

import { CipherService } from "@bitwarden/common/vault/abstractions/cipher.service";

@Injectable()
export class AutoTypeService {
  constructor(private cipherService: CipherService) {
    ipcRenderer.on("autotype-sortcut", (event, value) => {
      this.autotype(value);
    });
  }

  async autotype(appName: string) {
    const matchedUri = "windowsapp://" + appName;
    const matches = await this.cipherService.getAllDecryptedForUrl(matchedUri);

    if (matches.length > 0) {
      ipcRenderer.send(
        "autotype",
        matches[0].login.username || "",
        matches[0].login.password || ""
      );
    }
  }
}
