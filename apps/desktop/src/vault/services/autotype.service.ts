import { Injectable } from "@angular/core";
import { ipcRenderer } from "electron";

import { CipherService } from "@bitwarden/common/vault/abstractions/cipher.service";

@Injectable()
export class AutoTypeService {
  constructor(private cipherService: CipherService) {
    ipcRenderer.on("autotype:sortcut", (event, value) => {
      this.autotype(value);
    });
  }

  async autotype(url: string) {
    const matchedUri = url;
    const matches = await this.cipherService.getAllDecryptedForUrl(matchedUri);

    if (matches.length > 0) {
      ipcRenderer.send(
        "autotype:execute",
        matches[0].login.username || "",
        matches[0].login.password || ""
      );
    }
  }

  async nextWindow() {
    return await ipcRenderer.invoke("autotype:get_next_window");
  }
}
