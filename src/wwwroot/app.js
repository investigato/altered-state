const scenariosList = document.getElementById("scenariosList");

async function api(path, options = {}) {
    const response = await fetch(path, {
        headers: { "Content-Type": "application/json" },
        ...options,
    });

    let data = null;

    try {
        data = await response.json();
    } catch {
        data = { message: await response.text() };
    }

    if (!response.ok) {
        throw new Error(data?.message || data?.title || response.statusText);
    }
    if (!data.ok) {
        throw new Error(data?.message || data?.title || "Unknown error");
    }
    return data;
}

async function getActiveScenario() {
    try {
        const data = await api("/api/scenarios/active");
        return data?.active_scenario || "default";
        // return "gold";
    } catch (err) {

        return "Error fetching active scenario";
    }
}

async function listAvailableScenarios(activeName) {
    try {
        const data = await api("/api/scenarios");
        const scenarios = data?.scenarios || [];
        // sort so active scenario is first, then alphabetically by name
        scenarios.sort((a, b) => {
            if (a.name === activeName) return -1;
            if (b.name === activeName) return 1;
            return a.name.localeCompare(b.name);
        });
        scenariosList.innerHTML = "";
        scenarios.forEach((scenario) => {
            const isActive = scenario.name === activeName;
            const listItem = createListItem(scenario, isActive);
            scenariosList.appendChild(listItem);
        });
        addEventListeners();
    } catch (err) {
        scenariosList.innerHTML = "";
        const errorItem = document.createElement("li");
        errorItem.className =
            "col-span-1 flex flex-col divide-y divide-gray-200 rounded-lg bg-white text-center shadow-sm dark:divide-white/10 dark:bg-gray-800/50 dark:shadow-none dark:outline dark:-outline-offset-1 dark:outline-white/10";
        errorItem.textContent = `Error loading scenarios: ${err.message}`;
        scenariosList.appendChild(errorItem);
    }
}
// scenario: string & state: "baseline"
async function runAction(label, path, scenarioName) {
    const currentButtons = document.querySelectorAll(".action-btn");
    currentButtons.forEach((btn) => { btn.disabled = true; });
    const requestBody = {
        scenario: scenarioName,
    };
    try {
        await api(path, { method: "POST", body: JSON.stringify(requestBody) });
        const activeScenarioName = await getActiveScenario();
        await listAvailableScenarios(activeScenarioName);
    } catch (err) {
        console.error(`${label} action failed: ${err.message}`);
    }
    document.querySelectorAll(".action-btn").forEach((btn) => {
        btn.disabled = false;
    });
}

getActiveScenario().then((activeScenarioName) => {
    listAvailableScenarios(activeScenarioName).then(r => {
    });
});

function createListItem(scenario, isActive) {
    const name = scenario.name || "Unnamed Scenario";
    const description = scenario.description || "No description";
    const image =
        scenario.image ||
        "default.jpg";
    const imagePath = `./images/${image}`;
    const listItem = document.createElement("li");
    listItem.className =
        "col-span-1 flex flex-col divide-y divide-gray-200 rounded-lg bg-white text-center shadow-sm dark:divide-white/10 dark:bg-gray-800/50 dark:shadow-none dark:outline dark:-outline-offset-1 dark:outline-white/10";

    const contentDiv = document.createElement("div");
    contentDiv.className = "flex flex-1 flex-col p-4";

    const img = document.createElement("img");
    img.src = imagePath;
    img.alt = name;
    img.className =
        "mx-auto size-32 shrink-0 rounded-full bg-gray-300 outline -outline-offset-1 outline-black/5 dark:bg-gray-700 dark:outline-white/10";
    contentDiv.appendChild(img);

    const title = document.createElement("h3");
    title.className = "mt-6 text-sm font-medium text-gray-900 dark:text-white";
    title.textContent = name;
    contentDiv.appendChild(title);

    const descriptionEl = document.createElement("dl");
    descriptionEl.className = "mt-1 flex grow flex-col justify-between";

    const descriptionElementData = document.createElement("dd");
    descriptionElementData.className =
        "text-sm text-gray-500 dark:text-gray-400 mt-1";
    descriptionElementData.textContent = description;
    descriptionEl.appendChild(descriptionElementData);

    if (isActive === true) {

        const activeIndicatorScreenReader = document.createElement("dt");
        activeIndicatorScreenReader.className = "sr-only";
        activeIndicatorScreenReader.textContent = "Active Scenario";
        descriptionEl.appendChild(activeIndicatorScreenReader);
        const activeIndicator = document.createElement("dd");
        activeIndicator.className = "mt-3";
        const activeIndicatorBadge = document.createElement("span");
        activeIndicatorBadge.className =
            "inline-flex items-center rounded-full bg-green-50 px-2 py-1 text-xs font-medium text-green-700 inset-ring inset-ring-green-600/20 dark:bg-green-500/10 dark:text-green-500 dark:inset-ring-green-500/10";
        activeIndicatorBadge.textContent = "Active";
        activeIndicator.appendChild(activeIndicatorBadge);
        descriptionEl.appendChild(activeIndicator);
    }

    contentDiv.appendChild(descriptionEl);
    const bottomDiv = document.createElement("div");

    const bottomContentDiv = document.createElement("div");
    bottomContentDiv.className =
        "-mt-px flex divide-x divide-gray-200 dark:divide-white/10";

    const buttonWrapper = document.createElement("div");
    buttonWrapper.className = "flex w-0 flex-1";
    const actionButton = document.createElement("button");
    actionButton.type = "button";
    const buttonColorScheme = isActive
        ? "bg-rose-600 hover:bg-rose-500 focus-visible:outline-rose-600 dark:bg-rose-500 dark:hover:bg-rose-400 dark:focus-visible:outline-rose-500"
        : "bg-emerald-600 hover:bg-emerald-500 focus-visible:outline-emerald-600 dark:bg-emerald-500 dark:hover:bg-emerald-400 dark:focus-visible:outline-emerald-500";
    actionButton.className = `action-btn relative mr-0 inline-flex w-0 flex-1 items-center justify-center rounded-bl-lg rounded-br-lg border border-transparent py-4 text-sm font-semibold text-gray-900 dark:text-white px-3 shadow-xs focus-visible:outline-2 focus-visible:outline-offset-2 dark:shadow-none ${buttonColorScheme}`;
    actionButton.textContent = isActive ? "Reset" : "Activate";
    actionButton.setAttribute("data-id", scenario.name);
    actionButton.setAttribute(
        "data-action",
        isActive ? "reset" : "activate",
    );
    buttonWrapper.appendChild(actionButton);
    bottomContentDiv.appendChild(buttonWrapper);

    bottomDiv.appendChild(bottomContentDiv);
    listItem.appendChild(contentDiv);
    listItem.appendChild(bottomDiv);
    return listItem;
}

function addEventListeners() {
    const activateButtons = document.querySelectorAll(
        ".action-btn[data-action='activate']",
    );
    activateButtons.forEach((btn) => {
        btn.addEventListener("click", () => {
            const scenarioName = btn.getAttribute("data-id");
            runAction("Activate", `/api/scenarios/activate`, scenarioName);
        });
    });

    const resetButtons = document.querySelectorAll(
        ".action-btn[data-action='reset']",
    );
    resetButtons.forEach((btn) => {
        btn.addEventListener("click", () => {
            const scenarioName = btn.getAttribute("data-id");
            runAction("Reset", `/api/scenarios/reset`, scenarioName);
        });
    });
}
