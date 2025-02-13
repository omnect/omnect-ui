<script setup lang="ts">
import { ref } from "vue"
import { useRouter } from "vue-router"
import { useCentrifuge } from "../../composables/useCentrifugo"

const { token, initializeCentrifuge } = useCentrifuge()
const router = useRouter()

const username = ref("")
const password = ref("")
const visible = ref(false)

const doLogin = async (e: Event) => {
	e.preventDefault()

	const creds = btoa(`${username.value}:${password.value}`)

	const res = await fetch("token/login", {
		method: "POST",
		headers: {
			Authorization: `Basic ${creds}`
		}
	})

	if (res.ok) {
		const resToken = await res.text()
		token.value = resToken
		initializeCentrifuge()
		router.push({
			path: "/",
			force: true
		})
	}
}
</script>
  
<template>
    <v-card
      class="mx-auto pa-12 pb-8"
      elevation="8"
      max-width="448"
      rounded="lg"
    >
        <v-form @submit.prevent @submit="doLogin">
        <v-text-field
            label="Username"
            density="compact"
            placeholder="Username"
            prepend-inner-icon="mdi-account-outline"
            variant="outlined"
            v-model="username"
        ></v-text-field> 

        <v-text-field
            label="Password"
            :append-inner-icon="visible ? 'mdi-eye-off' : 'mdi-eye'"
            :type="visible ? 'text' : 'password'"
            density="compact"
            placeholder="Enter your password"
            prepend-inner-icon="mdi-lock-outline"
            variant="outlined"
            @click:append-inner="visible = !visible"
            v-model="password"
        ></v-text-field>

        <v-btn
            class="mb-8"
            color="secondary"
            size="large"
            variant="text"
            type="submit"
            block
        >
            Log In
        </v-btn>
        </v-form>
    </v-card>
</template>