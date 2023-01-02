document.addEventListener("DOMContentLoaded", function () {
    renderMathInElement(document.body, {
        delimiters: [
            { left: '$$', right: '$$', display: true },
            { left: '$', right: '$', display: false },
            { left: '\\(', right: '\\)', display: false },
            { left: '\\[', right: '\\]', display: true }
        ],
    });

    let theorems = Array.prototype.slice.call(document.getElementsByClassName("math-theorem"));
    let theoremCount = 1;
    for (let theorem of theorems) {
        let title = theorem.dataset.title || "";
        let content = theorem.innerHTML;
        theorem.classList.add("mdc-card");
        theorem.classList.add("mdc-card--outlined");
        theorem.classList.add("mdc-card__content");
        theorem.classList.add("block");
        theorem.classList.add("theorem");
        theorem.innerHTML = `
                <div class="theorem-title">
                    (定理 ${theoremCount}) ${title}
                </div>
                <div class="statement">
                    ${content}
                </div>`;
        theoremCount++;
    }

    let proofs = Array.prototype.slice.call(document.getElementsByClassName("math-proof"));
    for (let proof of proofs) {
        let content = proof.innerHTML;
        proof.classList.add("block");
        proof.classList.add("proof");
        proof.innerHTML = `
                <a class="proof-prefix">(証明)</a>
                <div class="proof-content">
                    ${content}
                    <span class="proof-endmark">□</span>
                </div>`;
    }

    let cases = Array.prototype.slice.call(document.getElementsByClassName("math-case"));
    for (let case_ of cases) {
        let content = case_.innerHTML;
        let condition = case_.classList.contains("otherwise") ? "それ以外の場合" : `<span class="rule">${case_.dataset.rule}</span> の場合`;
        case_.innerHTML = `
                <li class="case">
                    ${condition}
                    <p style="padding-left:1em">
                        ${content}
                    </p>
                </li>
                `;
    }

    let proofPrefixes = document.getElementsByClassName("proof-prefix");
    for (let proofPrefix of proofPrefixes) {
        proofPrefix.addEventListener("click", () => {
            let correspondProofContent = proofPrefix.parentElement.getElementsByClassName("proof-content")[0];
            if (!correspondProofContent) {
                return;
            }
            if (correspondProofContent.style.display == "none") {
                correspondProofContent.style.display = "block";
                proofPrefix.textContent = "(証明)";
            } else {
                correspondProofContent.style.display = "none";
                proofPrefix.textContent = "(証明) ...";
            }
        })
    }
});