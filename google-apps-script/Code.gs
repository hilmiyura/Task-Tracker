/**
 * Task Tracker — Google Apps Script Web App
 *
 * Menerima POST JSON dari aplikasi Task Tracker dan menuliskannya ke baris baru
 * pada tab sesuai "Sheet name" yang dikirim aplikasi. Pemetaan kolom:
 *   B = tanggal   (date)
 *   C = jam mulai (start)
 *   D = jam akhir (end)
 *   I = kegiatan  (task)
 *
 * Cara pasang / update:
 *  1. Buka Google Sheet target.
 *  2. Menu: Extensions > Apps Script.
 *  3. Hapus isi default, tempel seluruh kode ini, lalu Save.
 *  4. Deploy > New deployment > Type: Web app
 *       - Execute as: Me (akun Anda)
 *       - Who has access: Anyone
 *     (Jika sudah pernah deploy: Manage deployments > Edit > Version: New version
 *      agar perubahan kode aktif, URL tetap sama.)
 *  5. Authorize / Allow saat diminta.
 *  6. Salin "Web app URL" (berakhiran /exec) → tempel ke Settings (⚙︎) di app.
 */

// Script ini HARUS dibuat dari dalam sheet (Extensions > Apps Script) supaya
// getActiveSpreadsheet() menunjuk ke spreadsheet ini.
//
// Nama tab WAJIB dikirim aplikasi (field "sheet" dari Settings). Tidak ada
// default — kalau kosong, request ditolak.

// Pemetaan kolom (nomor kolom: B=2, C=3, D=4, I=9).
var COL_DATE = 2; // B
var COL_START = 3; // C
var COL_END = 4; // D
var COL_TASK = 9; // I

// Kolom "penanda" baris kosong & baris awal data.
// Sebuah baris dianggap belum terpakai jika kolom penanda ini kosong.
var ANCHOR_COL = COL_DATE; // B (Tanggal)
var START_ROW = 2; // data mulai baris 2 (baris 1 = header)

function doPost(e) {
  try {
    var data = JSON.parse(e.postData.contents);

    // Tab name is required — it comes from the app (Settings → Sheet name).
    var sheetName = data.sheet ? String(data.sheet).trim() : "";
    if (!sheetName) {
      return json({ ok: false, error: "Sheet name is required (set it in app Settings)" });
    }

    var ss = SpreadsheetApp.getActiveSpreadsheet();
    var sheet = ss.getSheetByName(sheetName);
    if (!sheet) {
      return json({ ok: false, error: 'Tab "' + sheetName + '" not found' });
    }

    var row = firstEmptyRow(sheet);

    sheet.getRange(row, COL_DATE).setValue(data.date || "");
    sheet.getRange(row, COL_START).setValue(data.start || "");
    sheet.getRange(row, COL_END).setValue(data.end || "");
    sheet.getRange(row, COL_TASK).setValue(data.task || "");

    return json({ ok: true, row: row });
  } catch (err) {
    return json({ ok: false, error: String(err) });
  }
}

/**
 * Cari baris pertama (mulai START_ROW) yang kolom penanda (ANCHOR_COL) kosong.
 * Hanya melihat satu kolom, jadi rumus/nilai statis di kolom lain tidak
 * mempengaruhi. Jika semua terisi, pakai baris setelah yang terakhir terisi.
 */
function firstEmptyRow(sheet) {
  var maxRows = sheet.getMaxRows();
  if (maxRows < START_ROW) return START_ROW;

  var values = sheet
    .getRange(START_ROW, ANCHOR_COL, maxRows - START_ROW + 1, 1)
    .getValues();

  for (var i = 0; i < values.length; i++) {
    if (values[i][0] === "" || values[i][0] === null) {
      return START_ROW + i;
    }
  }
  // Semua baris terisi → tulis di baris baru di bawahnya (sheet auto-extend).
  return maxRows + 1;
}

// Opsional: cek cepat di browser bahwa Web App hidup.
function doGet() {
  return json({ ok: true, service: "Task Tracker webhook" });
}

function json(obj) {
  return ContentService.createTextOutput(JSON.stringify(obj)).setMimeType(
    ContentService.MimeType.JSON
  );
}
