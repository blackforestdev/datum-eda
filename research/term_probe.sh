#!/usr/bin/env bash
# Datum terminal probe — run DIRECTLY in a Datum shell prompt (not via Claude).
pause(){ printf '\n\e[2m-- %s -- press Enter --\e[0m' "$1"; read -r _; }

printf '\e[1m1. SGR\e[0m: \e[1mbold\e[0m \e[2mdim\e[0m \e[3mitalic\e[0m \e[4munder\e[0m \e[4:3mcurly\e[0m \e[7mrev\e[0m \e[9mstrike\e[0m \e[53moverline\e[0m \e[1;4;31mcombo\e[0m\n'
printf '16c: '; for i in 30 31 32 33 34 35 36 37 90 91 92 93 94 95 96 97; do printf "\e[${i}m██"; done; printf '\e[0m\n'
printf 'bg : '; for i in 40 41 42 43 44 45 46 47 100 101 102 103 104 105 106 107; do printf "\e[${i}m  "; done; printf '\e[0m\n'
printf '256: '; for i in $(seq 16 6 231); do printf "\e[48;5;${i}m "; done; printf '\e[0m\n'
printf 'RGB: '; for i in $(seq 0 3 255); do printf "\e[48;2;${i};$((255-i));128m "; done; printf '\e[0m\n'
pause "expect: styles distinct, 3 smooth color ramps"

printf '\e[1m2. Cursor addressing\e[0m\n'
printf '\e[s'; for r in 1 2 3; do printf "\e[${r}B\e[10G row+%d col10" "$r"; printf '\e[u'; done; printf '\e[4B\r'
printf '\e[3A\e[30Cup3-right30\e[3B\r'
printf 'save/restore + \e[K erase-EOL test: XXXXXXXXXX\e[10D\e[Kclean\n'
printf 'insert/delete: abcdef\e[6D\e[3@---\e[6C\e[2P\n'
pause "expect: rows labelled, 'clean' with no X, 'ab---cdef' then 'ab---cd'"

printf '\e[1m3. Scroll region\e[0m (rows 5-10 scroll, header stays)\n'
printf '\e[2J\e[H\e[7m HEADER — must not scroll \e[0m\n\e[5;10r\e[5;1H'
for i in $(seq 1 25); do printf 'scrolling line %d\n' "$i"; sleep 0.03; done
printf '\e[r\e[12;1H'
pause "expect: header fixed, lines 5-10 scrolled, last = 25"

printf '\e[1m4. Alt screen\e[0m\n'
printf '\e[?1049h\e[H\e[2J\e[5;10H\e[1;33m*** ALT SCREEN — this must vanish on return ***\e[0m\e[7;10Hpress Enter'; read -r _
printf '\e[?1049l'
printf 'back on main screen — the ALT text should be gone and this line follows section 3.\n'
pause "expect: main screen restored intact"

printf '\e[1m5. Wide/combining/emoji\e[0m\n'
printf '|日本語|中文|한국어|\n|é ñ ü|👨‍👩‍👧|🇺🇸|\n|\xe2\x80\x8b<-ZWSP|\xef\xbc\xa1FULLWIDTH|\n'
printf 'wrap at edge: '; printf '日%.0s' $(seq 1 60); printf '\n'
pause "expect: aligned bars, no half-cells, wide char never split at right edge"

printf '\e[1m6. Bracketed paste\e[0m: enable, now PASTE something (Ctrl+Shift+V) then Enter\n'
printf '\e[?2004h'; IFS= read -r line; printf '\e[?2004l'
printf 'received: %q\n' "$line"
pause "expect: starts with \$'\\\\E[200~' and ends '\\\\E[201~'"

printf '\e[1m7. Mouse (SGR)\e[0m: click/scroll in the pane for 5s\n'
printf '\e[?1000h\e[?1006h'; stty -echo -icanon; timeout 5 cat -v; stty sane; printf '\e[?1006l\e[?1000l\n'
pause "expect: ^[[<0;col;rowM on press, ^[[<0;col;rowm release, 64/65 wheel"

printf '\e[1m8. Terminal queries\e[0m\n'
stty -echo -icanon min 0 time 5
for pair in '\e[c|DA1' '\e[>c|DA2' '\e[6n|CPR' '\e[5n|DSR' '\e[?u|kitty-query' '\e[18t|size(t18)' '\e[?2004$p|DECRQM 2004' '\eP+q544e\e\\|XTGETTCAP TN'; do
  seq=${pair%%|*}; name=${pair##*|}
  printf "$seq"; resp=$(dd bs=128 count=1 2>/dev/null | cat -v); printf '  %-16s -> %s\n' "$name" "$resp"
done
stty sane
pause "expect: DA1/DA2/CPR/DSR answered; kitty-query & DECRQM may be silent (P15)"

printf '\e[1m9. Title + bell + hyperlink\e[0m\n'
printf '\e]0;DATUM PROBE TITLE\a'; printf '\a'; printf '\e]8;;https://example.com\e\\OSC8 link\e]8;;\e\\  |  '; printf '\e]52;c;%s\a' "$(printf 'clip-test' | base64)"; printf 'OSC52 clipboard set to "clip-test"\n'
pause "expect: tab title changed, bell indicator, link styled, clipboard=clip-test"

printf '\e[1m10. Throughput\e[0m: 200k lines\n'; time (seq 1 200000)
printf '\nDone. Now try TUIs: htop, vim (:set mouse=a), less, and resize the pane mid-output.\n'
