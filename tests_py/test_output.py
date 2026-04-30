from __future__ import annotations

import io
import unittest

from prospectus_graphicus.output import Format, render_rows


class OutputTests(unittest.TestCase):
    def test_render_rows_json(self) -> None:
        out = io.StringIO()
        render_rows(out, Format.JSON, [{"id": "1", "subject": "Hello"}], lambda row: row["subject"])
        self.assertIn('"subject": "Hello"', out.getvalue())


if __name__ == "__main__":
    unittest.main()
