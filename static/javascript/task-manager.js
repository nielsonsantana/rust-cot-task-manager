function taskApp() {
    return {
        isAuthenticated: Alpine.$persist(false),
        email: '',
        userEmail: '',
        otpSent: false,
        otpCode: '',
        tasks: [],
        newTaskTitle: '',
        filter: 'All',

        async init(){
            if(this.isAuthenticated){
                await this.checkSession();
            }
        },

        async checkSession() {
            let res = await fetch(window.AppConfig.urls.get_current_user);
            if (res.ok) {
                let data = await res.json();
                this.userEmail = data.email;
                await this.fetchTasks();
            } else {
                this.isAuthenticated = false;
                this.userEmail = '';
            }
        },

        async sendOtp() {
            if (!this.email) return;
            await fetch(window.AppConfig.urls.send_otp, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ email: this.email })
            });
            this.otpSent = true;
        },

        async verifyOtp() {
            let res = await fetch(window.AppConfig.urls.verify_otp, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ email: this.email, code: this.otpCode })
            });
            if (res.ok) {
                let data = await res.json();
                this.isAuthenticated = true;
                this.userEmail = data.username;
                await this.fetchTasks();
            } else {
                let err_msg = await res.text();
                alert(err_msg || window.AppConfig.i18n.invalid_code);
            }
        },

        async logout() {
            try {
                await fetch(window.AppConfig.urls.logout, { method: 'GET' });
            } catch (e) {
                console.error("Logout request failed:", e);
            }
            this.isAuthenticated = false;
            this.otpSent = false;
            this.email = '';
            this.otpCode = '';
            this.userEmail = '';
            this.tasks = [];
            location.href = "/";
        },

        async fetchTasks() {
            let res = await fetch(window.AppConfig.urls.list_tasks);
            this.tasks = await res.json();
        },

        get filteredTasks() {
            if (this.filter === 'All') return this.tasks;
            return this.tasks.filter(t => t.status === this.filter);
        },

        async createTask() {
            if (!this.newTaskTitle.trim()) return;
            await fetch(window.AppConfig.urls.create_task, {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ title: this.newTaskTitle })
            });
            this.newTaskTitle = '';
            await this.fetchTasks();
        },

        async updateTaskStatus(id, status) {
            const targetUrl = window.AppConfig.urls.update_task.replace("__ID__", encodeURIComponent(id));
            await fetch(targetUrl, {
                method: 'PATCH', 
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify({ status })
            });
            await this.fetchTasks();
        },

        async deleteTask(id) {
            const targetUrl = window.AppConfig.urls.delete_task.replace("__ID__", encodeURIComponent(id));
            await fetch(targetUrl, { method: 'DELETE' });
            await this.fetchTasks();
        }
    }
}