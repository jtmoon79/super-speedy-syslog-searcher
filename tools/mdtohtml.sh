#!/usr/bin/env bash
#
# run `mdtohtml` with preferred options
#
#     go install github.com/gomarkdown/mdtohtml@latest
#

set -eu

cd "$(dirname -- "${0}")/.."

# in case `mdtohtml` is not in the PATH, add the default Go bin directory to the PATH
export PATH="${PATH}:${HOME}/go/bin"

file=${1}
file_html="${file}.html"
shift

# make tables more readable
css_snippet=$(cat <<'CSS'
table {  border-collapse: collapse; width: 100%;}

th,
td {
  border: 1px solid #666;
  padding: 0.5rem 0.75rem;
  text-align: left;
}

th {
  background: #e9ecef;
  font-weight: 700;
  border-bottom: 2px solid #333;
}
CSS
)

# make tables sortable
js_snippet=$(cat <<'JS'
document.addEventListener('DOMContentLoaded', function () {
  var columnTypes = ['alpha', 'numeric', 'alpha', 'numeric', 'numeric', 'numeric'];

  function numericValue(text) {
    var match = String(text).replace(/,/g, '').match(/-?[0-9]+(?:[.][0-9]+)?/);
    return match ? Number(match[0]) : NaN;
  }

  function compareCells(aText, bText, type, asc) {
    if (type === 'numeric') {
      var aNum = numericValue(aText);
      var bNum = numericValue(bText);
      var aNaN = Number.isNaN(aNum);
      var bNaN = Number.isNaN(bNum);

      if (aNaN && bNaN) {
        return 0;
      }
      if (aNaN) {
        return 1;
      }
      if (bNaN) {
        return -1;
      }
      return asc ? aNum - bNum : bNum - aNum;
    }

    var cmp = String(aText).localeCompare(String(bText), undefined, {
      numeric: true,
      sensitivity: 'base'
    });
    return asc ? cmp : -cmp;
  }

  function getRowsForSort(table) {
    if (table.tBodies && table.tBodies.length > 0) {
      return {
        container: table.tBodies[0],
        rows: Array.from(table.tBodies[0].rows)
      };
    }

    var allRows = Array.from(table.rows);
    return {
      container: table,
      rows: allRows.slice(1)
    };
  }

  Array.from(document.querySelectorAll('table')).forEach(function (table) {
    var headerRow = table.tHead && table.tHead.rows.length > 0
      ? table.tHead.rows[0]
      : table.rows[0];

    if (!headerRow) {
      return;
    }

    Array.from(headerRow.cells).forEach(function (th, colIndex) {
      th.style.cursor = 'pointer';
      th.addEventListener('click', function () {
        var rowsInfo = getRowsForSort(table);
        var rows = rowsInfo.rows;
        var sortType = columnTypes[colIndex] || 'alpha';
        var currentlyAsc = th.getAttribute('data-sort-dir') === 'asc';
        var asc = !currentlyAsc;

        Array.from(headerRow.cells).forEach(function (cell) {
          if (cell !== th) {
            cell.removeAttribute('data-sort-dir');
          }
        });

        rows.sort(function (aRow, bRow) {
          var aCell = aRow.cells[colIndex];
          var bCell = bRow.cells[colIndex];
          var aText = aCell ? aCell.textContent.trim() : '';
          var bText = bCell ? bCell.textContent.trim() : '';
          return compareCells(aText, bText, sortType, asc);
        });

        rows.forEach(function (row) {
          rowsInfo.container.appendChild(row);
        });

        th.setAttribute('data-sort-dir', asc ? 'asc' : 'desc');
      });
    });
  });
});
JS
)

echo "${PS4}mdtohtml -page ${file} ${*}" >&2

# convert MD to HTML and use `awk` to embed CSS and JavaScript midstream
# for writing into `$file_html`
mdtohtml -page "$file" "${@}" | \
  awk -v css="${css_snippet}" -v js="${js_snippet}" '
  BEGIN {
    style = "<style>\n" css "\n</style>"
    script = "<script>\n" js "\n</script>"
    injected = style "\n" script
    inserted = 0
  }
  {
    if (!inserted && $0 ~ /<\/[Hh][Ee][Aa][Dd]>/) {
      print injected
      inserted = 1
    }
    print
  }
  END {
    if (!inserted) {
      print injected
    }
  }
' > "${file_html}"
