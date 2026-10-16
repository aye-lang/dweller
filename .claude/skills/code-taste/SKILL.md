---
name: code-taste
description: CEO 的 code taste 規則集（Aye 版，源自 anyknown-com 的 code-taste）。寫或改 browser/ 的任何程式碼之前先讀；派 sub agent 寫 code 時把相關條目原文貼進 prompt；CEO 抱怨 code 難看時，把抱怨收斂成一條新規則加進來。Triggers on code taste, 重構, refactor, 難看, clean up, 寫 code, review。
---

# code-taste

CEO 看 code 會崩潰的東西，一條一條記在這裡。每條：規則、壞例、好例。規則來自真實抱怨，不憑空發明。條目從 anyknown-com 的 code-taste 翻成 Aye；只適用 TypeScript/React 的條目（zod、設計系統、`useEffect`）不搬。

## 流程（CEO 抱怨時）

1. 把抱怨收斂成一條可檢查的規則（一句話 + 壞例 / 好例），加進下面「規則」。跟既有條目重疊就改那條，不新增。
2. `rg` 找出全 repo 同類違規，按模組（`browser/<module>/`）分批。
3. 每批一張 T 卡（`docs/plans/taste.md`），派一個 opus sub agent（並行要 `isolation: worktree`）。只排沒有進行中卡片碰的模組。prompt 必含：規則全文與壞 / 好例、檔案清單、「只改這條規則涵蓋的東西，行為不變」、驗證指令（該模組的 targeted tests + `python3 browser/tools/check_taste.py`）、交回 commit hash + 測試結果。
4. 主 session 抽查 diff 與測試結果，不重讀全部。
5. 能機械檢查的規則進 `browser/tools/check_taste.py`（CI 跑），不靠人記。舊檔用 baseline 棘輪：只准變好，不准變壞；新檔一律照規則。

## 規則

### 1. 不寫註釋

專注在易讀性與架構：命名、檔案切分、函數切分表達意思。想寫註釋 = 該抽函數、改名或拆檔。
唯一例外：不得不連到外部資料，上限一行：spec 的 URL，或參考實作的路徑（`// v8: src/interpreter/bytecode-generator.cc`、`// blink: core/layout/...`，見 docs 的 reference 規則）。
檔頭說明、`///` 敘述、段落分隔線、解釋「為什麼」的多行註釋全部是違規。
所有權與「可能沒有」用型別和名字講：`?T` 表示可能沒有；擁有的切片叫 `owned_*` 或由 `deinit` 的那個 struct 持有，不靠欄位註釋。
架構層級的「為什麼」寫進 `docs/*.md` 的設計文件，不寫在 code 裡。

壞：

```aye
// Lower case, without the `:`.
scheme: []u8
```

好：

```aye
scheme_lower: []u8
```

### 2. 邏輯進模組，殼只接線

`main.aye` 與 `ui/` 的視窗迴圈只做 wiring；邏輯放在它所屬的模組。內嵌的 JavaScript（`engine/prelude.aye` 這類）放在 `.js` 檔，用 `@embedFile` 讀進來，不寫成 Aye 字串字面量。

### 3. 檔案 ≤ 300 行

產生的表（`tools/gen_*.py` 的輸出）與 vendored 檔除外。

### 4. 最少的 code

不為單次使用做抽象、不加沒人要的彈性、不處理不可能發生的錯誤。

### 5. 一個檔案一件事；模組是目錄

Aye 的模組本來就是目錄，同一目錄的檔案共用一個命名空間，所以拆檔不改任何 import。

一個檔案只放一個函數，加上只服務它的一兩行小 helper。檔案裡出現第二個有份量的函數、常數表、型別群，就拆出去：

```
url/
├── url.aye               struct Url 與它的欄位
├── url-error.aye
├── parse.aye
├── resolve.aye
├── serialize.aye
├── percent-encode.aye
└── default-port.aye
```

硬門檻：一個非測試檔不准有兩個以上超過 5 行的函數。「它很 cohesive」不是豁免。
豁免只有五種：一個 struct 與它的方法（方法仍受 300 行限制：太長就把方法按主題拆到同目錄的其他檔，若語言不允許跨檔方法，記成 compiler gap）、一個 enum 與只服務它的 `match` 表、純資料表、只接線的 registry（native 函數表、opcode 表：handler 各自一檔）、產生的檔。
同前綴的檔案家族升級成子模組目錄：`canvas_api.aye canvas_prelude.aye` → `canvas/`。
派 sub agent 時把這段原文貼進去：不寫死門檻，agent 會用「已經是一件事」躲掉九成的檔。

檔名 = 函數名或型別名的 kebab-case（Aye 編譯器若不接受 `-`，就用 snake_case，並在第一張 T 卡記下結論）。Aye 單元測試 `*_test.aye` 貼著它測的檔。

### 6. 先找現成的，才准自己寫

順序：Aye `std` → 已經連結的 C 函式庫（SDL、FreeType、HarfBuzz、libcurl、zlib、ANGLE、wasmtime）→ 新的 C 函式庫（MIT / BSD / Apache 授權）→ 自己寫。自己寫之前要能說出現成的為什麼不行。
瀏覽器引擎本身的演算法照參考實作移植（V8、ANGLE 可移植程式碼；Blink 只參考演算法），不憑空發明。

### 7. 留空行，讓 code 呼吸

函數本體裡，不同性質的東西之間空一行：

- 一組單行 `let` / `var` 之後空一行；
- 多行的宣告（函數、struct 字面量、多行呼叫）前後各空一行；
- `return` 之前空一行（它是區塊裡唯一一句時除外）；
- `if` / `for` / `while` / `match` 這種區塊語句前後各空一行；
- 頂層的函數、常數、型別之間各空一行。

連續的單行 `let` 彼此不用空行；連續的單行 guard（`if ... { return ... }`）彼此不用空行。

壞：

```aye
let start = self.pos
let end = self.scanName()
if end == start {
    return null
}
return self.src[start..end]
```

好：

```aye
let start = self.pos
let end = self.scanName()

if end == start {
    return null
}

return self.src[start..end]
```

由 `check_taste.py` 擋（之後若 `aye fmt` 支援，改由它處理）。
