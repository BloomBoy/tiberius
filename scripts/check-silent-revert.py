#!/usr/bin/env python3
"""Fall nar en commit skriver tillbaka nagon annans nyss tillagda rader.

TRE GANGER PA ETT DYGN, 12-13 september 2026:

  * 6e43e8ea0 "fix(katalog): Flux-fixturen far K7:s tre falt" bar 80 filer och
    skrev tillbaka 52 av dem till aldre versioner. K7 och K4 forsvann ur main
    och tva sessioner letade i utrullningen efter kod som inte fanns.
  * 05638f599 "feat(objects): metrics under snippets" rorde fyra adapterfiler
    och tog bort fyra evidensfixar som lagts in en timme tidigare.
  * En tredje, samma kvall, upptackt forst nar en avbild redan var utrullad.

Monstret ar alltid detsamma: en session arbetar i ett trad som star pa en aldre
main, kor `git add -A` eller `git commit-tree`, och far med sig GAMLA versioner
av filer som nagon annan andrat under tiden. Commiten ser liten ut i sin
rubrik. `merge-base --is-ancestor` sager "ja, fixen finns" - om COMMITEN, inte
om raderna.

Ingen grind fangade det, for varje enskild commit kompilerar och alla tester
gar igenom: koden som forsvann var ny och hade inga egna gamla tester som foll.

DEN HAR STALLER EN ANNAN FRAGA: tar commiten bort rader som nagon LADE TILL i
de senaste N commitarna? Da ar det antingen en avsiktlig atergang - som ska
sagas i commit-texten - eller den har olyckan.

    python3 scripts/check-silent-revert.py [<commit>] [--depth N]

Grон = inga tillbakaskrivna rader, eller commit-texten sager "revert".
"""
import re
import subprocess
import sys

DEPTH = 40


def sh(*args: str) -> str:
    return subprocess.run(["git", *args], capture_output=True, text=True, check=False).stdout


def is_merge(commit: str) -> bool:
    return len(sh("rev-list", "--parents", "-n", "1", commit).split()) > 2


def diff_args(commit: str) -> list[str]:
    """Hur man far fram VAD en commit gjorde — aven nar den ar en merge.

    `git show` pa en merge visar INGENTING alls som forval. Forsta versionen av
    den har filen sa darfor "OK: HEAD ror inga filer" om en merge som forde in
    tolv andrade filer — gron pa en tom mangd, dygnets felfamilj, i sparren som
    skrevs for att fanga just den familjen.

    Mot forsta foraldern: det ar vad mergen tillforde main.
    """
    if is_merge(commit):
        return ["diff", "--unified=0", f"{commit}^1", commit]
    return ["show", "--format=", "--unified=0", commit]


def added_lines(commit: str, path: str) -> set[str]:
    """Raderna den commiten LA TILL i filen, utan diff-prefixet."""
    out = sh(*diff_args(commit), "--", path)
    return {
        line[1:].strip()
        for line in out.splitlines()
        if line.startswith("+") and not line.startswith("+++") and line[1:].strip()
    }


def removed_lines(commit: str, path: str) -> set[str]:
    out = sh(*diff_args(commit), "--", path)
    return {
        line[1:].strip()
        for line in out.splitlines()
        if line.startswith("-") and not line.startswith("---") and line[1:].strip()
    }


# En atergang ska DEKLARERAS, inte namnas. Tva former racker, och bada ar
# nagot en manniska skriver med avsikt:
#
#   `Revert "…"`          git egen form
#   `revert(scope): …`    typen forst, som ovriga commit-rubriker i repot;
#                         Jira-nyckeln forst ar tillaten (`TOAD-65: revert(spec): …`),
#                         eftersom varje rubrik ska borja med den (16 oktober)
#   `… avsiktlig atergang …`  formuleringen byggledaren ber om
#
# Det som INTE langre racker ar att ordet forekommer nagonstans i texten. Den
# 19 september godkandes commiten "docs(byggledare): revertsparren godkanner en
# merge…" av sin egen beskrivning av felet: en grind som slapper igenom allt
# som SKRIVER OM den ar ingen grind.
DECLARED = re.compile(
    r"""^(TOAD-\d+[:,]?\s+)*Revert\s+"          # git: Revert "…", optionally after the Jira keys
      | ^(TOAD-\d+[:,]?\s+)*(revert|rollback|restore)(\([^)]*\))?!?:   # [TOAD-N: ]revert(scope)!: …
      | \b(avsiktlig|medveten)\s+
        (atergang|återgång|aterstallning|återställning|rollback)\b
    """,
    re.I | re.X,
)


def declares_revert(subject: str) -> bool:
    """Sager DEN HAR commiten sjalv att den ar en avsiktlig atergang?

    Bara sin egen rubrik. Forsta versionen la till andra foralderns rubrik nar
    den provade en merge, med det riktiga skalet att en merge bar en
    intetsagande egen rubrik ("Merge branch …"). Foljden var att en gren kunde
    godkanna sig sjalv: sa toppcommiten "atergang" slapptes HELA mergen
    igenom, och de ovriga commitarna provades aldrig. Merge 7426d6a93 forde in
    nio commits pa den grunden (19 september) — det var i sin ordning den
    gangen, men av tur, inte av grind.

    Mergens rubrik skriver byggledaren. Det ar ratt stalle for pastaendet,
    eftersom det ar byggledaren som svarar for vad som fors in i main.
    """
    return bool(DECLARED.search(subject))


def main() -> int:
    args = [a for a in sys.argv[1:] if not a.startswith("--")]
    commit = args[0] if args else "HEAD"
    depth = DEPTH
    if "--depth" in sys.argv:
        depth = int(sys.argv[sys.argv.index("--depth") + 1])

    subject = sh("log", "-1", "--format=%s", commit).strip()
    parents = sh("rev-list", "--parents", "-n", "1", commit).split()[1:]
    # ALLA inforda grenar, inte bara den forsta av dem: en oktopusmerge har
    # flera, och en sparre som laser `parents[1]` ensam skulle namna fel gren i
    # sin forklaring utan att saga att den valde. Vi gor inga oktopusmergar,
    # men en grind ska falla tydligt pa det den inte raknat med, inte tyst.
    branch_tips = [sh("log", "-1", "--format=%s", p).strip() for p in parents[1:]]
    if declares_revert(subject):
        print(f"OK: {commit[:9]} sager sjalv att den ar en atergang")
        return 0

    if is_merge(commit):
        files = [f for f in sh("diff", "--name-only", f"{commit}^1", commit).splitlines() if f]
    else:
        files = [f for f in sh("show", "--format=", "--name-only", commit).splitlines() if f]
    if not files:
        print(f"OK: {commit[:9]} ror inga filer")
        return 0

    # Commitarna FORE den har, i samma historik.
    earlier = [c for c in sh("log", "--format=%H", f"{commit}~1", f"-{depth}").splitlines() if c]

    findings: list[str] = []
    for path in files:
        gone = removed_lines(commit, path)
        if not gone:
            continue
        for prev in earlier:
            # Hoppa over commitens egna foraldrar i en merge: en merge tar inte
            # bort nagot, den forenar.
            put_there = added_lines(prev, path)
            if not put_there:
                continue
            overlap = gone & put_there
            # En enstaka rad kan vara en aterkommande `}` eller en tom rad som
            # rakar matcha. Tre eller fler ar ett monster, inte ett sammantraffande.
            if len(overlap) >= 3:
                who = sh("log", "-1", "--format=%an: %s", prev).strip()
                findings.append(
                    f"  {path}\n"
                    f"    {len(overlap)} rader som {prev[:9]} la till ar borttagna igen\n"
                    f"    den commiten: {who}"
                )
                break

    if not findings:
        print(f"OK: {commit[:9]} skriver inte tillbaka nagon annans nya rader")
        return 0

    print(f"TILLBAKASKRIVNING i {commit[:9]} — \"{subject}\"\n")
    print("\n".join(findings))
    claiming = [t for t in branch_tips if declares_revert(t)]
    if claiming:
        vilka = "\n".join(f'    "{t}"' for t in claiming)
        gren = "Grenens toppcommit sager" if len(claiming) == 1 else \
               f"{len(claiming)} av de inforda grenarnas toppcommits sager"
        print(
            f"\n  {gren} atergang:\n{vilka}\n"
            "  — men mergens EGEN rubrik gor inte det, och det ar den som galler\n"
            "  har. En gren kan inte godkanna sig sjalv: skriv 'avsiktlig\n"
            "  atergang' i mergens rubrik om det ar det du for in, sa star\n"
            "  pastaendet dar den som mergar satte det."
        )
    print(
        "\nEn commit som tar bort rader nagon annan nyss la in ar antingen en\n"
        "avsiktlig atergang - skriv da 'revert' i commit-texten - eller ett trad\n"
        "som stod pa en aldre main och committades med `-A`. Det senare har hant\n"
        "tre ganger pa ett dygn och kostat tva utrullningar.\n"
        "\nAtgard: aterstall filerna ur den commit som la in dem,\n"
        "  git checkout <den commiten> -- <fil>\n"
        "en fil i taget, aldrig ett helt trad."
    )
    return 1


if __name__ == "__main__":
    raise SystemExit(main())
