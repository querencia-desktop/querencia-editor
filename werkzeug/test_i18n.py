#!/usr/bin/env python3
"""Tests für werkzeug/i18n.py: doppelte Zugriffstasten je Dialog/Menü.

Aufruf im Repo-Wurzelverzeichnis: python3 -I werkzeug/test_i18n.py
"""

import contextlib
import io
import os
import sys
import unittest
import warnings

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import i18n  # noqa: E402

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))


class Zugriffstaste(unittest.TestCase):
    def test_taste_hinter_dem_ersten_unterstrich(self):
        self.assertEqual(i18n.mnemonic_key("Ö_ffnen …"), "f")
        self.assertEqual(i18n.mnemonic_key("Am Ende _neu beginnen"), "n")
        self.assertEqual(i18n.mnemonic_key("_Suchen"), "s")
        self.assertEqual(i18n.mnemonic_key("Ohne Taste"), None)
        self.assertEqual(i18n.mnemonic_key("doppelt__unterstrichen"), None)


class Doppelungen(unittest.TestCase):
    GRUPPE = {"Dialog": [(None, "_Find"), (None, "_Replace"), ("ctx", "_Close")]}
    KNOWN = {(None, "_Find"), (None, "_Replace"), ("ctx", "_Close")}

    def test_eindeutige_tasten_ohne_meldung(self):
        cat = {"de": {(None, "_Find"): "_Suchen", (None, "_Replace"): "_Ersetzen",
                      ("ctx", "_Close"): "Schl_ießen"}}
        self.assertEqual(i18n.mnemonic_clashes(self.KNOWN, cat, self.GRUPPE), [])

    def test_doppelte_taste_wird_gemeldet(self):
        # Stand 0.2.2: S_chließen / Su_chen
        cat = {"de": {(None, "_Find"): "Su_chen", (None, "_Replace"): "_Ersetzen",
                      ("ctx", "_Close"): "S_chließen"}}
        fehler = i18n.mnemonic_clashes(self.KNOWN, cat, self.GRUPPE)
        self.assertEqual(len(fehler), 1, fehler)
        self.assertIn("Zugriffstaste C doppelt in Dialog", fehler[0])
        self.assertIn("'S_chließen'", fehler[0])
        self.assertIn("'Su_chen'", fehler[0])

    def test_gross_und_klein_gelten_gleich(self):
        cat = {"de": {(None, "_Find"): "_Suchen", (None, "_Replace"): "Er_setzen",
                      ("ctx", "_Close"): "Schl_ießen"}}
        fehler = i18n.mnemonic_clashes(self.KNOWN, cat, self.GRUPPE)
        self.assertEqual(len(fehler), 1, fehler)
        self.assertIn("Zugriffstaste S doppelt", fehler[0])

    def test_quellsprache_wird_geprueft(self):
        known = {(None, "_Find"), (None, "_File")}
        cat = {"msgid": {k: k[1] for k in known}}
        fehler = i18n.mnemonic_clashes(known, cat, {"Menü": sorted(known, key=str)})
        self.assertEqual(len(fehler), 1, fehler)
        self.assertTrue(fehler[0].startswith("msgid: Zugriffstaste F doppelt"), fehler)

    def katalog_mit(self, ersatz):
        """de.po des Repos, einzelne Übersetzungen ersetzt: {(ctxt, msgid): msgstr}."""
        _, entries = i18n.read_po(os.path.join(ROOT, "po", "de.po"))
        cat = {(e["ctxt"], e["id"]): e["str"] for e in entries if e.get("plural") is None}
        cat.update(ersatz)
        known = {(e["ctxt"], e["id"]) for e in i18n.extract(ROOT)}
        return i18n.mnemonic_clashes(known, {"de": cat})

    def test_stand_022_wird_gemeldet(self):
        # Katalog 0.2.0 bis 0.2.2: S_chließen/Su_chen (C), Am _Ende/_Ersetzen (E)
        fehler = self.katalog_mit({
            ("Search dialog", "_Close"): "S_chließen", (None, "_Find"): "Su_chen",
            (None, "Wra_p around"): "Am _Ende neu beginnen", (None, "_Search for:"): "_Suchen nach:",
        })
        self.assertEqual(sorted(f.split(":")[1].strip() for f in fehler),
                         ["Zugriffstaste C doppelt in Dialog Ersetzen (src/search.rs)",
                          "Zugriffstaste C doppelt in Dialog Suchen (src/search.rs)",
                          "Zugriffstaste E doppelt in Dialog Ersetzen (src/search.rs)"], fehler)

    def test_feld_gegen_knopf_im_selben_dialog(self):
        # Knopf „_Suchen“, aber das Feld noch „_Suchen nach:“ (beide S)
        fehler = self.katalog_mit({(None, "_Search for:"): "_Suchen nach:"})
        self.assertEqual(len(fehler), 2, fehler)
        for f in fehler:
            self.assertIn("Zugriffstaste S doppelt", f)
            self.assertIn("'_Suchen nach:'", f)
            self.assertIn("'_Suchen'", f)

    def test_unbekannte_msgid_wird_gemeldet(self):
        fehler = i18n.mnemonic_clashes(self.KNOWN, {}, {"Dialog": [(None, "_Weg")]})
        self.assertEqual(fehler, ["Zugriffstasten Dialog: unbekannte msgid '_Weg'"])

    def test_alle_gruppen_im_katalog_bekannt(self):
        known = {(e["ctxt"], e["id"]) for e in i18n.extract(ROOT)}
        for name, keys in i18n.MNEMONIC_GROUPS.items():
            for k in keys:
                self.assertIn(k, known, name)

    def test_repo_besteht_pruefen(self):
        # i18n.py liest Dateien mit open(…).read(); unter unittest wären das
        # nur Warnungen über nicht geschlossene Dateien.
        warnings.simplefilter("ignore", ResourceWarning)
        out = io.StringIO()
        with contextlib.redirect_stdout(out):
            rc = i18n.cmd_pruefen(ROOT)
        self.assertEqual(rc, 0, out.getvalue())
        self.assertNotIn("doppelt", out.getvalue())


if __name__ == "__main__":
    unittest.main()
